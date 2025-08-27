use crate::clients::client_chen::{ClientChen, NodeInfo, PacketResponseHandler, Router, Sending, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::routing_algorithms::dijkstra::DijkstraRouting;
use crate::clients::client_chen::routing_algorithms::routing_trait::shortest_path_with_algorithm;
use crate::general_use::PacketStatus::{Sent, WaitingForFixing};

impl PacketResponseHandler for ClientChen {
    fn handle_ack(&mut self, ack_packet_session_id: SessionId, ack: &Ack) {
        /*println!(
            "\n==============================================\n\
         ✔ ACK RECEIVED\n\
         ─────────────────────────────────────────────\n\
         Session ID    : {}\n\
         Fragment Index: {}\n\
         =============================================\n",
            ack_packet_session_id, ack.fragment_index
        );*/
        let session_id = ack_packet_session_id;
        let fragment_index = ack.fragment_index;

        // Update packets_status using nested HashMap access
        self.update_packet_status(session_id, fragment_index, Sent);

        // Remove from output_buffer using proper nested structure
        if let Some(fragments) = self.storage.output_buffer.get_mut(&session_id) {
            fragments.remove(&fragment_index);
        }
    }


    fn handle_nack(&mut self, nack_packet: Packet, nack: &Nack) {
        // Handle specific NACK types
        match nack.nack_type.clone() {
            NackType::ErrorInRouting(node_id) => self.handle_error_in_routing(node_id, nack_packet.session_id, nack),
            NackType::DestinationIsDrone => self.handle_destination_is_drone(nack_packet.session_id, nack),
            NackType::Dropped => self.handle_packet_dropped(nack_packet, nack),
            NackType::UnexpectedRecipient(node_id) => self.handle_unexpected_recipient(node_id, nack_packet.session_id, nack),
        }
    }


    fn handle_error_in_routing(&mut self, node_id: NodeId, nack_packet_session_id: SessionId, nack: &Nack) {
        // Clean up packet_send connection
        if self.communication_tools.packet_send.remove(&node_id).is_some() {
            warn!("Removed broken connection to node {} from packet_send", node_id);
        }

        println!("-------NACK--------");
        println!("Nack: routing error encountered in drone {}: crashed or sender not found", node_id);

        //remove the drone from the topology, and also from the connected_ids of routers that are connected to it.
        //the ideal is to only remove from the connected_ids of the router before it, but since we don't know the implementation
        //of the sending process of the nack of each drone, we just simplify.

        let connected_nodes = if let Some(node_info) = self.network_info.topology.get(&node_id) {
            node_info.connected_nodes_ids.clone()  // Clone to avoid holding reference
        } else {
            HashSet::new()  // Return empty vec if source node not found
        };

        // Now iterate and mutate (mutable borrow - separate from immutable borrow)
        for node_id_to_update in connected_nodes {
            if let Some(node_info) = self.network_info.topology.get_mut(&node_id_to_update) {
                node_info.connected_nodes_ids.remove(&node_id);
            }
        }

        self.network_info.topology.remove(&node_id);

        let session_id = nack_packet_session_id;
        let fragment_index = nack.fragment_index;

        self.update_packet_status(
            session_id,
            fragment_index,
            PacketStatus::NotSent(NotSentType::RoutingError(node_id)),
        );

        let opt_packet = self.storage.output_buffer
            .get_mut(&session_id)
            .and_then(|fragments| fragments.get_mut(&fragment_index))
            .cloned();

        let option_packet_to_send = {
            if let Some(mut packet) = opt_packet {
                println!("Need to resend the packet with same session id: {}", packet.session_id);
                let opt_destination = packet.routing_header.destination();
                if let Some(destination) = opt_destination {

                    //send by calculating with dijkstra algorithm
                    let path = shortest_path_with_algorithm(&DijkstraRouting, self.metadata.node_id, destination, &self.network_info.topology);
                    println!("But first get routing through Dijkstra algorithm");

                    if let Some(path) = path{
                        println!("Dijkstra Path: {:?}", path);
                        packet.routing_header = SourceRoutingHeader::initialize(path);
                        Some(packet.clone())
                    } else{
                        None
                    }
                } else {
                    None // Packet to send
                }
            } else {
                warn!("Packet not found in output buffer (Session: {}, Fragment: {})", session_id, fragment_index);
                None
            }
        };

        // Send the packet when conditions are satisfied
        if let Some(p) = option_packet_to_send {
            // Notice that by sending, it will automatically update the PacketStatus
            self.send(p);
            println!("Packet of session {} resent", session_id);
        }

        println!("-------END NACK--------");
    }
    fn handle_destination_is_drone(&mut self, nack_packet_session_id: SessionId, nack: &Nack) {
        let session_id = nack_packet_session_id;
        self.update_packet_status(session_id, nack.fragment_index, PacketStatus::NotSent(NotSentType::DroneDestination));
        self.storage.output_buffer.remove(&(session_id));  //we don't want anymore this packet.
    }
    fn handle_packet_dropped(&mut self, nack_packet: Packet, nack: &Nack) {
        println!("Packet of session {} is dropped from drone {}", nack_packet.session_id, nack_packet.routing_header.source().unwrap());

        let session_id = nack_packet.session_id;

        // When the drone pdr is very high then we need to fix, we give him chance up to 10 times repeating pack drop.
        /*if let Some(drone) = nack_packet.routing_header.source() {
            let map = self
                .communication
                .drops_counter
                .entry(session_id)
                .or_insert_with(HashMap::new);

            let counter = map.entry(drone).or_insert(0);
            *counter += 1;

            if *counter == 10 {
                *counter = 0;
                let me = (self.metadata.node_id, NodeType::Client);
                self.send_event(ClientEvent::CallTechniciansToFixDrone(drone, me));

                self.storage
                    .packets_status
                    .entry(session_id)
                    .or_insert_with(HashMap::new)
                    .insert(nack.fragment_index, WaitingForFixing(drone));

                return;
            }
        }*/


        let prev_cost:f32;
        // Augment the costs in Dijkstra algorithm
        if let Some(drone) = nack_packet.routing_header.source() {
            if let Some(node_info) = self.network_info.topology.get_mut(&drone) {
                let mut pdr : f32 = 0.0;
                if let SpecificInfo::DroneInfo(drone_info) = &mut node_info.specific_info {
                    drone_info.dropped_count += 1;
                    pdr = drone_info.dropped_count as f32 / drone_info.sent_count as f32;
                }
                prev_cost = node_info.routing_cost;
                node_info.routing_cost = 1f32/(1f32 - pdr.max(0.0).min(0.99));
                println!("Cost of the drone {} increased from {} to {}", drone, prev_cost, node_info.routing_cost);
            }

        }
        self.update_packet_status(
            session_id,
            nack.fragment_index,
            PacketStatus::NotSent(NotSentType::Dropped),
        );

        let opt_packet = self.storage.output_buffer
            .get_mut(&session_id)
            .and_then(|fragments| fragments.get_mut(&nack.fragment_index))
            .cloned();

        let option_packet_to_send = {
            if let Some(mut packet) = opt_packet {
                println!("Need to resend the packet with same session id: {}", packet.session_id);
                let opt_destination = packet.routing_header.destination();
                if let Some(destination) = opt_destination {

                    //send by calculating with dijkstra algorithm
                    let path = shortest_path_with_algorithm(&DijkstraRouting, self.metadata.node_id, destination, &self.network_info.topology);
                    println!("But first get routing through Dijkstra algorithm");

                    if let Some(path) = path{
                        println!("Dijkstra Path: {:?}", path);
                        packet.routing_header = SourceRoutingHeader::initialize(path);
                        Some(packet.clone())
                    } else{
                        None
                    }
                } else {
                    None // Packet to send
                }
            } else {
                warn!("Packet not found in output buffer (Session: {}, Fragment: {})", session_id, nack.fragment_index);
                None
            }
        };

        // Send the packet when conditions are satisfied
        if let Some(p) = option_packet_to_send {
            // Notice that by sending, it will automatically update the PacketStatus
            self.send(p);
            println!("Packet of session {} resent", session_id);
        }

        /*println!(
            "\n*******************************************************************\n\
        🚁 Dropped packet resent\n\
        -------------------------------------------------------------------\n\
        📊 Packet status:\n{:#?}\n\
        *******************************************************************",
            self.storage.packets_status
        );*/
    }

    fn handle_unexpected_recipient(&mut self, node_id: NodeId, nack_packet_session_id: SessionId, nack: &Nack) {
        info!("unexpected recipient found {}", node_id);
        let session_id = nack_packet_session_id;
        let fragment_index = nack.fragment_index;

        self.update_packet_status(
            session_id,
            nack.fragment_index,
            PacketStatus::NotSent(NotSentType::BeenInWrongRecipient(node_id)));

        let opt_packet = self.storage.output_buffer
            .get_mut(&session_id)
            .and_then(|fragments| fragments.get_mut(&fragment_index))
            .cloned();


        let option_packet_to_send = {
            if let Some(mut packet) = opt_packet {
                println!("DEBUGGING SESSION ID: {}", packet.session_id);
                let opt_destination = packet.routing_header.destination();
                if let Some(destination) = opt_destination {
                    let path = shortest_path_with_algorithm(&DijkstraRouting, self.metadata.node_id, destination, &self.network_info.topology);
                    if let Some(path) = path{
                        packet.routing_header = SourceRoutingHeader::initialize(path);
                        Some(packet.clone())
                    } else{
                        None
                    }
                } else {
                    None // Packet to send
                }
            } else {
                warn!("Packet not found in output buffer (Session: {}, Fragment: {})", session_id, fragment_index);
                None
            }
        };

        // Send the packet when conditions are satisfied
        if let Some(p) = option_packet_to_send {
            // Notice that by sending, it will automatically update the PacketStatus
            self.send(p);
            println!("DEBUGGING PACKET SESSION ID SENT: {}", session_id);
        }
    }
}