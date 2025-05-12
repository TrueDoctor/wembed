use std::cmp::max;
use std::fs::read_to_string;
use std::io;
use std::io::Write;

use crate::vec::DVec;

// A node in the graph
// Each node has a weight, which is degree ^ (d/8)
#[derive(Clone, Copy)]
pub struct Node {
    pub weight: f64,
}

// A graph structure
// It contains the embedding dimension, nodes, and edges
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<(usize, usize)>,
}

pub struct Embedding<'a, const D: usize> {
    pub positions: Vec<DVec<D>>,
    pub graph: &'a Graph,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
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
        for i in 0..node_degree.len() {
            graph.nodes.push(Node {
                // weight = degree ^ (d/8)
                weight: (node_degree[i] as f64).powf(embedding_dim as f64 / latent_dim_hint as f64)
                    * (node_degree.len() as f64 / total_weight as f64),
            });
        }
        Ok(graph)
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
}
