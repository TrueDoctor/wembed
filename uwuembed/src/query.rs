use crate::{graph::Embedding, vec::DVec};

pub trait Query {
    fn repelling_nodes(&self, index: usize) -> Vec<usize>;
    fn attracting_nodes(&self, index: usize) -> Vec<usize>;
}
pub trait Update<const D: usize> {
    fn update_positions(&mut self, postions: &[DVec<D>]);
}

pub trait Embedder<const D: usize>: Query + Update<D> {
    fn calculate_step(&mut self, dt: f64) {}
}

pub struct Naive<'a, const D: usize> {
    embedding: Embedding<'a, D>,
}

impl<'a, const D: usize> Naive<'a, D> {
    pub fn new(embedding: &'a Embedding<'a, D>) -> Self {
        Self {
            embedding: embedding.clone(),
        }
    }
}

impl<const D: usize> Query for Naive<'_, D> {
    fn repelling_nodes(&self, index: usize) -> Vec<usize> {
        let mut output = Vec::new();
        let graph = self.embedding.graph;
        let positions = &self.embedding.positions;
        let own_weight = graph.nodes[index].weight;
        let own_position = positions[index];

        for (i, (node, position)) in graph.nodes.iter().zip(positions.iter()).enumerate() {
            let weight = own_weight * node.weight;
            let distance = own_position.distance_squared(position);
            if distance < weight.powi(2) && !graph.is_connected(index, i) {
                output.push(i);
            }
        }
        output
    }

    fn attracting_nodes(&self, index: usize) -> Vec<usize> {
        let mut output = Vec::new();
        // let graph = self.embedding.graph;
        // let positions = &self.embedding.positions;
        // let own_weight = graph.nodes[index].weight;
        // let own_position = positions[index];

        // for (i, (node, position)) in graph.nodes.iter().zip(positions.iter()).enumerate() {
        //     if own_position.distance(position) / (own_weight * node.weight).powf(1. / D as f64) < 1.
        //         && graph.is_connected(index, i)
        //     {
        //         output.push(i);
        //     }
        // }
        output
    }
}
impl<'a, const D: usize> Update<D> for Naive<'a, D> {
    fn update_positions(&mut self, postions: &[DVec<D>]) {
        self.embedding.positions = postions.to_vec();
    }
}
