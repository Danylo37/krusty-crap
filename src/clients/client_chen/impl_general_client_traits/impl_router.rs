use crate::clients::client_chen::{ClientChen, NodeInfo, PacketCreator, Router, Sending, ServerInformation, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::SpecificInfo::ServerInfo;
use crate::general_use::PacketStatus::Sent;
use crate::general_use::ServerType::{Undefined, WaitingForResponse};

impl Router for ClientChen {
    ///main method of for discovering the routing
    fn do_flooding(&mut self) {
        // New ids for the flood and new session because of the flood response packet
        self.status.flood_id += 1;
        self.status.session_id += 1;

        self.network_info.topology.clear();

        // Initialize the flood request with the current flood_id, id, and node type
        let flood_request = FloodRequest::initialize(self.status.flood_id, self.metadata.node_id, NodeType::Client);

        // Prepare the packet with the current session_id and flood_request
        let packet = Packet::new_flood_request(
            SourceRoutingHeader::empty_route(),
            self.status.session_id,
            flood_request,
        );

        self.update_connected_nodes();
        let connected_nodes = self.communication.connected_nodes_ids.clone();
        //println!("|Web| Client [{}] connected_nodes: {:?}", self.metadata.node_id, connected_nodes);

        for &node_id in connected_nodes.iter() {
            //println!("|Web| Client [{}] sent to: {}", self.metadata.node_id, node_id);
            self.send_packet_to_connected_node(node_id, packet.clone()); // Assuming `send_packet_to_connected_node` takes a cloned packet
            self.update_packet_status(packet.session_id, 0, Sent);
        }
    }


    ///auxiliary function
    fn get_flood_response_initiator(&mut self, flood_response: FloodResponse) -> NodeId {
        flood_response.path_trace.last().map(|(id, _)| *id).unwrap()
    }

    fn update_topology_entry_for_server(&mut self, initiator_id: NodeId, server_type: ServerType) {
        match self.network_info.topology.entry(initiator_id) {
            Entry::Occupied(mut entry) => {
                let node_info = entry.get_mut();
                match &mut node_info.specific_info {
                    SpecificInfo::ServerInfo(server_info) => {
                        server_info.server_type = server_type;  // Update server type correctly
                    }
                    _ => {
                        // If the node exists but is not a ServerInfo, replace it
                        node_info.specific_info = SpecificInfo::ServerInfo(ServerInformation {
                            server_type,
                        });
                    }
                }
            }
            Entry::Vacant(entry) => {
                // If the node doesn't exist, insert it as a new ServerInfo
                entry.insert(NodeInfo {
                    node_id: initiator_id,
                    connected_nodes_ids: Default::default(),
                    routing_cost: 1.0,
                    specific_info: SpecificInfo::ServerInfo(ServerInformation {
                        server_type,
                    }),
                });
            }
        }
    }

    fn send_query_to_server_if_needed(&mut self, destination_id: NodeId, path_trace: Vec<(NodeId, NodeType)>) {
        let hops = self.get_hops_from_path_trace(path_trace);
        let srh = SourceRoutingHeader::initialize(hops);
        self.status.session_id += 1;
        let should_send_query = {
            if let Some(node_info) = self.network_info.topology.get_mut(&destination_id) {
                if let ServerInfo(server_info) = &mut node_info.specific_info {
                    if matches!(server_info.server_type, Undefined) {
                        server_info.server_type = WaitingForResponse;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };
        if should_send_query {
            self.send_query_by_routing_header(srh, Query::AskType);
        }
    }

    fn send_query_to_client_if_needed(&mut self, destination_id: NodeId, path_trace: Vec<(NodeId, NodeType)>) {
        let hops = self.get_hops_from_path_trace(path_trace);
        let srh = SourceRoutingHeader::initialize(hops);
        self.status.session_id += 1;
        // You can add logic here if you want to send a query to the client, e.g.:
        // self.send_query_by_routing_header(srh, Query::AskClientInfo);
        // For now, this is a placeholder for future client-specific queries.
    }

}



