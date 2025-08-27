use crate::clients::client_chen::prelude::*;
use std::collections::{HashMap};
use crate::clients::client_chen::NodeInfo;

pub trait RoutingAlgorithm {
    fn calculate_routes(
        &self,
        source: NodeId,
        topology: &HashMap<NodeId, NodeInfo>,
    ) -> HashMap<NodeId, Vec<NodeId>>;

    /// Calculates the shortest path from source to destination, stopping as soon as the destination is found.
    fn calculate_single_route(
        &self,
        source: NodeId,
        destination: NodeId,
        topology: &HashMap<NodeId, NodeInfo>,
    ) -> Option<Vec<NodeId>>;
}

pub fn shortest_path_with_algorithm(
    algorithm: &dyn RoutingAlgorithm,
    source: NodeId,
    destination: NodeId,
    topology: &HashMap<NodeId, NodeInfo>,
) -> Option<Vec<NodeId>> {
    algorithm.calculate_single_route(source, destination, topology)
}


