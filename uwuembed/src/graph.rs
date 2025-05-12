use std::cmp::max;
use std::fs::read_to_string;
use std::io;
use std::io::Write;

use crate::NodeId;
use crate::vec::DVec;

// A node in the graph
// Each node has a weight, which is degree ^ (d/8)
#[derive(Clone, Debug)]
pub struct Node {
    pub weight: f64,
    pub neighbors: Vec<usize>,
}
const DEG_PRECOMPUTE: usize = 200;

// A graph structure
// It contains the embedding dimension, nodes, and edges
#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<(NodeId, NodeId)>,
    pub pow_lut: [f64; DEG_PRECOMPUTE * DEG_PRECOMPUTE],
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Embedding<'a, const D: usize> {
    pub positions: Vec<DVec<D>>,
    pub graph: &'a Graph,
}

impl Graph {
    pub const fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
            pow_lut: [0.; DEG_PRECOMPUTE * DEG_PRECOMPUTE],
        }
    }

    /// Parses a graph from an edge list file.
    /// The file should contain pairs of integers representing edges.
    pub fn parse_from_edge_list_file(
        file_path: &str,
        embedding_dim: usize,
        latent_dim_hint: usize,
    ) -> io::Result<Self> {
        let mut graph = Graph::new();
        graph.edges = read_to_string(file_path)
            .unwrap()
            .lines()
            .map(|s| {
                s.split_ascii_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .map(|s| {
                let u = s[0].parse::<usize>().unwrap();
                let v = s[1].parse::<usize>().unwrap();
                (u, v)
            })
            .collect::<Vec<_>>();
        let mut node_degree = Vec::new();
        for (u, v) in graph.edges.iter() {
            if node_degree.len() <= max(*u, *v) {
                node_degree.resize(max(*u, *v) + 1, 0);
            }
            node_degree[*u] += 1;
            node_degree[*v] += 1;
        }
        let total_weight: usize = node_degree.iter().sum();
        let weight_norm = node_degree.len() as f64 / total_weight as f64;
        let dim_ratio = embedding_dim as f64 / latent_dim_hint as f64;
        for i in 0..node_degree.len() {
            graph.nodes.push(Node {
                // weight = degree ^ (d/8)
                weight: (node_degree[i] as f64).powf(dim_ratio) * weight_norm,
                neighbors: Vec::new(),
            });
        }
        for i in 0..DEG_PRECOMPUTE {
            for j in 0..DEG_PRECOMPUTE {
                graph.pow_lut[i * DEG_PRECOMPUTE + j] = ((i as f64).powf(dim_ratio)
                    * weight_norm
                    * (j as f64).powf(dim_ratio)
                    * weight_norm)
                    .powf(1. / embedding_dim as f64)
                    .powi(2)
            }
        }
        for (u, v) in graph.edges.iter() {
            graph.nodes[*u].neighbors.push(*v);
            graph.nodes[*v].neighbors.push(*u);
        }

        // TODO: Sort nodes by degree and reassign indices
        Ok(graph)
    }

    #[inline(always)]
    pub fn distance_weight_squared(&self, i: usize, j: usize, dimension_factor: f64) -> f64 {
        let deg_i = self.nodes[i].neighbors.len();
        let deg_j = self.nodes[j].neighbors.len();
        if deg_i >= DEG_PRECOMPUTE || deg_j >= DEG_PRECOMPUTE {
            // dbg!(deg_i.max(deg_j));
            // unsafe { unreachable_unchecked() };
            return (self.nodes[i].weight * self.nodes[j].weight).powf(dimension_factor);
        }
        self.pow_lut[deg_i * DEG_PRECOMPUTE + deg_j]
    }
}

// Debug functions

impl Graph {
    // Stores radii for each node when querying power of two weight classes
    pub fn save_radii_file(&self, file_path: &str, dimension: usize) {
        let mut file = std::fs::File::create(file_path).unwrap();
        write!(file, "Node, radius\n").unwrap();
        for i in 0..10 {
            for (j, node) in self.nodes.iter().enumerate() {
                //We set the radius to ri(u) = l · (w(u)*2^i)^(1/d)
                let radius =
                    (node.weight as f64 * 2.0_f64.powi(i as i32)).powf(1.0 / dimension as f64);
                writeln!(file, "{}, {}", j, radius).unwrap();
            }
        }
    }

    pub fn save_weights_file(&self, file_path: &str) {
        let mut file = std::fs::File::create(file_path).unwrap();
        write!(file, "Node, Degree\n").unwrap();
        for (i, node) in self.nodes.iter().enumerate() {
            writeln!(file, "{}, {}", i, node.weight).unwrap();
        }
    }

    pub fn is_connected(&self, u: usize, v: usize) -> bool {
        self.nodes[u].neighbors.contains(&v)
    }
}
