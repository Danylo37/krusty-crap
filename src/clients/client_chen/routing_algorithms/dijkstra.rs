use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use crate::clients::client_chen::{NodeId, NodeInfo};
use crate::clients::client_chen::routing_algorithms::routing_trait::RoutingAlgorithm;

pub struct DijkstraRouting;

impl RoutingAlgorithm for DijkstraRouting {
    fn calculate_routes(
        &self,
        source: NodeId,
        topology: &HashMap<NodeId, NodeInfo>,
    ) -> HashMap<NodeId, Vec<NodeId>> {
        use ordered_float::OrderedFloat;

        #[derive(Eq, PartialEq)]
        struct State {
            cost: OrderedFloat<f32>,  // Wrap f32 here
            position: NodeId,
        }

        // Your implementations remain the same
        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.cmp(&self.cost)  // Now this works!
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut distances: HashMap<NodeId, f32> = HashMap::new();
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        for &node in topology.keys() {
            distances.insert(node, f32::MAX);
        }
        distances.insert(source, 0f32);
        heap.push(State { cost: OrderedFloat::from(0f32), position: source });

        while let Some(State { cost, position }) = heap.pop() {  //pop the node with minimum distance, because the distances are only going to grow.
            if cost > OrderedFloat::from(distances[&position]) {
                continue;
            }
            let neighbors = topology.get(&position)
                .map(|info| info.connected_nodes_ids.clone())
                .unwrap_or_default();

            for &neighbor in &neighbors {  //push the neighbors and relative distances into the heap (if distances are smaller than previously saved)
                let mut cost_neighbor: f32 = 0f32;
                if let Some(node_info) = topology.get(&neighbor){
                    cost_neighbor = node_info.routing_cost;
                }
                let next = State { cost: cost + cost_neighbor, position: neighbor };
                if next.cost < OrderedFloat::from(*distances.get(&neighbor).unwrap_or(&f32::MAX)) {
                    distances.insert(neighbor, *next.cost);
                    previous.insert(neighbor, position); //the predecessor of the neighbor is the current node.
                    heap.push(next);
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
        let routing_table = self.calculate_routes(source, topology);
        if let Some(route) = routing_table.get(&destination){
            Some(route.clone())
        }
        else {
            None
        }
    }

}

