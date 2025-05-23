use crate::{
    Embedding, Naive, NodeId, Query,
    graph::{self, Graph},
    vec::DVec,
};
use rstar::RTree;
use rstar::primitives::GeomWithData;
use rstar::{Point, PointDistance};
use rstar::{RStarInsertionStrategy, RTreeParams};

type PointPosition<const D: usize> = GeomWithData<[f64; D], usize>;

// find weight class i such that 2^(i-1) <= weight < 2^i
fn compute_weight_class(graph: &Graph, index: usize) -> usize {
    let weight = graph.nodes[index].neighbors.len() as f64;
    let mut i = 0;
    while (1 << i) as f64 <= weight {
        i += 1;
    }
    i
}

struct LargeNodeParameters;

impl RTreeParams for LargeNodeParameters {
    const MIN_SIZE: usize = 10;
    const MAX_SIZE: usize = 100;
    const REINSERTION_COUNT: usize = 40;
    type DefaultInsertionStrategy = RStarInsertionStrategy;
}

#[derive(Debug)]
pub struct WRTree<'a, const D: usize> {
    pub positions: Vec<DVec<D>>,
    graph: &'a Graph,
    pub rtrees: Vec<RTree<PointPosition<D>, LargeNodeParameters>>,
    max_weights: Vec<f64>,
}

impl<'a, const D: usize> WRTree<'a, D> {
    pub fn new(embedding: &'a Embedding<'a, D>) -> Self {
        let rtrees = Vec::new();
        let max_weights = Vec::new();
        Self {
            positions: embedding.positions.to_vec(),
            graph: embedding.graph,
            rtrees,
            max_weights,
        }
    }

    pub fn update_positions(&mut self, positions: &[DVec<D>]) {
        let mut weight_classes = Vec::new();
        let mut max_weights: Vec<f64> = Vec::new();
        for (i, position) in positions.iter().enumerate() {
            self.positions[i] = *position;
            let weight_class = compute_weight_class(self.graph, i);
            if weight_class >= weight_classes.len() {
                weight_classes.resize(weight_class + 1, Vec::new());
                max_weights.resize(weight_class + 1, 0.0);
            }
            weight_classes[weight_class].push(i);
            max_weights[weight_class] =
                f64::max(max_weights[weight_class], self.graph.nodes[i].weight);
        }
        self.max_weights = max_weights;
        self.rtrees.clear();
        for (i, weight_class) in weight_classes.iter().enumerate() {
            self.rtrees.push(
                RTree::<PointPosition<D>, LargeNodeParameters>::bulk_load_with_params(
                    weight_class
                        .iter()
                        .map(|&index| PointPosition::new(positions[index].components, index))
                        .collect::<Vec<_>>(),
                ),
            );
        }
        println!("RTree updated with {} weight classes", self.rtrees.len());
        for (i, weight_class) in weight_classes.iter().enumerate() {
            println!(
                "Weight class {}: {} nodes, max weight: {}",
                i,
                weight_class.len(),
                self.max_weights[i]
            );
        }
    }

    /* for (i, (node, position)) in graph.nodes.iter().zip(positions.iter()).enumerate() {
           let weight = own_weight * node.weight;
           let distance = own_position.distance_squared(position);
           if distance < weight.powi(2)
           /*&& !graph.is_connected(index, i)*/
           {
               output.push(i);
           }
       }
    */
    pub fn repelling_nodes(&self, index: usize) -> Vec<usize> {
        let own_position = self.positions[index].components;
        let own_weight = self.graph.nodes[index].weight;
        let mut result = Vec::new();

        for weight_class in 0..self.rtrees.len() {
            // Calculate the radius based on the maximum weight of the class
            let radius = (own_weight * self.max_weights[weight_class] as f64).powi(2);

            result.extend(
                self.rtrees[weight_class]
                    .locate_within_distance(own_position, radius)
                    .map(|node| node.data)
                    .collect::<Vec<_>>(),
            );
        }
        result.retain(|&node| {
            // Check the distance to the own node
            let own_weight = self.graph.nodes[index].weight;
            let own_position = self.positions[index];
            let distance = own_position.distance_squared(&self.positions[node]);
            let weight = own_weight * self.graph.nodes[node].weight;
            distance < weight.powi(2)
        });
        result
    }
}
