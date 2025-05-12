use crate::{NodeId, graph::Graph, vec::DVec};

#[derive(Debug, Clone)]
struct UwuTree<'a, const D: usize> {
    positions: Vec<DVec<D>>,
    graph: &'a Graph,
    spatial_neighbors_lists: Vec<Vec<NodeId>>,
    clusters: Vec<Cluster<D>>,
}

#[derive(Debug, Clone)]
struct Cluster<const D: usize> {
    members: Vec<NodeId>,
    position: DVec<D>,
    radius: f64,
}

impl<'a, const D: usize> UwuTree<'a, D> {
    fn generate_clusters(&self, weight_threshold: f64) -> Vec<NodeId> {
        let mut clusters = Vec::new();
        for (i, (node, pos)) in self
            .graph
            .nodes
            .iter()
            .zip(self.positions.iter())
            .enumerate()
        {
            if node.weight > weight_threshold {
                clusters.push(Cluster {
                    members: Vec::new(),
                    position: *pos,
                    radius: 0.,
                })
            }
        }

        todo!()
    }

    fn compute_weight_threshold(&self, num: usize) -> f64 {
        let mut weights = self.graph.nodes.clone();
        weights.sort_unstable_by_key(|n| -(n.neighbors.len() as isize));
        weights[num.min(weights.len())].weight
    }
}
