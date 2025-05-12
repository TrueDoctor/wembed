use crate::graph::Embedding;

trait Query {
    fn repelling_nodes_with_radius(&self, index: usize, radius: f64) -> Vec<usize>;
    fn repelling_nodes_with(&self, index: usize) -> Vec<usize> {
        self.repelling_nodes_with_radius(index, 1.)
    }
}

struct Naive<'a, const D: usize> {
    embedding: Embedding<'a, D>,
}

impl<const D: usize> Query for Naive<'_, D> {
    fn repelling_nodes_with_radius(&self, index: usize, radius: f64) -> Vec<usize> {
        let mut output = Vec::new();
        let graph = self.embedding.graph;
        let positions = &self.embedding.positions;
        let own_weight = graph.nodes[index].weight;
        let own_position = positions[index];

        for (i, (node, position)) in graph.nodes.iter().zip(positions.iter()).enumerate() {
            if own_position.distance(position) / (own_weight * node.weight).powf(1. / D as f64)
                < radius
            {
                output.push(i);
            }
        }
        output
    }
}
