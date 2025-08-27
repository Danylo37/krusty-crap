use crate::clients::client_chen::{ClientChen, ClientInformation, DroneInformation, FloodingPacketsHandler, NodeInfo, Router, Sending, ServerInformation, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::general_use::PacketStatus::Sent;

impl FloodingPacketsHandler for ClientChen {
    fn handle_flood_request(&mut self, session_id: SessionId, request: &mut FloodRequest) {
        //println!("{:?} Client {} has received flood request that contains the path: {:?}", self.metadata.client_type ,self.metadata.node_id , request.path_trace);
        // Prepare the flood response.
        request.increment(self.metadata.node_id, self.metadata.node_type);
        let response = request.generate_response(session_id);

        //you send directly because the source routing header is there
        self.send(response.clone());
        self.update_packet_status(response.session_id, 0, Sent);
    }

    /// When you receive a flood response, you need first to update the topology with the elements of the path_traces
    /// everyone's connected_node_ids (using the hashset's methods).
    fn handle_flood_response(&mut self, response: &FloodResponse) {
        // Debugging: Print the received path trace
        //println!("{:?} Client {} has received flood response with the path: {:?}", self.metadata.client_type ,self.metadata.node_id , response.path_trace);
        // Check if path_trace is empty
        if response.path_trace.is_empty() {
            info!("ERROR: path_trace is empty!");
            return;
        }

        // Ensure the flood_id matches
        if response.flood_id != self.status.flood_id {
            return;
        }

        // Update the network topology
        let mut path_iter = response.path_trace.iter().peekable(); //we make the vector peekable
        let mut previous_node: Option<NodeId> = None;

        while let Some(&(node_id, node_type)) = path_iter.next() {
            // Peek the next node in the path_trace (use the item without consuming it)
            let next_node = path_iter.peek().map(|&(next_id, _)| next_id);
            // Ensure entry exists for the node, so create a raw one when it is not created for the node
            let node_info = self.network_info.topology.entry(node_id).or_insert_with(|| {
                match node_type {
                    NodeType::Server => {
                        NodeInfo {
                            node_id,
                            connected_nodes_ids: HashSet::new(),
                            routing_cost: 1.0,
                            specific_info: SpecificInfo::ServerInfo(ServerInformation {
                                server_type: ServerType::Undefined,
                            }),
                        }
                    },
                    NodeType::Client => NodeInfo {
                        node_id,
                        connected_nodes_ids: HashSet::new(),
                        routing_cost: 1.0,
                        specific_info: SpecificInfo::ClientInfo(ClientInformation {
                        }),
                    },
                    NodeType::Drone => NodeInfo {
                        node_id,
                        connected_nodes_ids: HashSet::new(),
                        routing_cost: 1.0,
                        specific_info: SpecificInfo::DroneInfo(DroneInformation {
                            dropped_count: 0,
                            sent_count: 0,
                        }),
                    },
                }
            });

            // Safely update connected_nodes_ids
            if let Some(prev) = previous_node {
                node_info.connected_nodes_ids.insert(prev);
            }
            if let Some(&next) = next_node {
                node_info.connected_nodes_ids.insert(next);
            }

            // Update previous_node safely
            previous_node = Some(node_id);
        }

        // Update routing table
        if let Some((destination_id, destination_type)) = response.path_trace.last().copied() {
            if destination_type == NodeType::Drone || response.flood_id != self.status.flood_id {
                return;
            }
            // Use match to call the correct update function
            match destination_type {
                NodeType::Server => {
                    self.send_query_to_server_if_needed(destination_id, response.path_trace.clone());
                }
                NodeType::Client => {
                    self.send_query_to_client_if_needed(destination_id, response.path_trace.clone());
                }
                _ => {}
            }
        }
    }

}