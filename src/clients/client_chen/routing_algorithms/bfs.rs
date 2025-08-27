use std::collections::{HashMap, HashSet, VecDeque};
use crate::clients::client_chen::{NodeId, NodeInfo};
use crate::clients::client_chen::routing_algorithms::routing_trait::RoutingAlgorithm;

pub struct BfsRouting;
impl RoutingAlgorithm for BfsRouting {
    fn calculate_routes(
        &self,
        source: NodeId,
        topology: &HashMap<NodeId, NodeInfo>,
    ) -> HashMap<NodeId, Vec<NodeId>> {
    
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(source);
        queue.push_back(source);

        while let Some(current) = queue.pop_front() {
            let neighbors = topology.get(&current)
                .map(|info| info.connected_nodes_ids.clone())
                .unwrap_or_default();
            for &neighbor in &neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    previous.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }

        let mut routing_table = HashMap::new();
        for &dest in topology.keys() {
            if dest == source { continue; }
            let mut path = Vec::new();
            let mut current = dest;
            while let Some(&prev) = previous.get(&current) {
                path.push(current);
                if prev == source { path.push(source); break; }
                current = prev;
            }
            path.reverse();
            if !path.is_empty() && path[0] == source {
                routing_table.insert(dest, path);
            }
        }
        routing_table
    }

    fn calculate_single_route(
        &self,
        source: NodeId,
        destination: NodeId,
        topology: &HashMap<NodeId, NodeInfo>,
    ) -> Option<Vec<NodeId>> {
        if source == destination {
            return Some(vec![source]);
        }
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(source);
        queue.push_back(source);
        while let Some(current) = queue.pop_front() {
            let neighbors = topology.get(&current)
                .map(|info| info.connected_nodes_ids.clone())
                .unwrap_or_default();
            for &neighbor in &neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    previous.insert(neighbor, current);
                    if neighbor == destination {
                        // Reconstruct path
                        let mut path = vec![destination];
                        let mut curr = current;
                        while curr != source {
                            path.push(curr);
                            curr = previous[&curr];
                        }
                        path.push(source);
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }
}
