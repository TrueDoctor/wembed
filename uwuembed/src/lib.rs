pub use graph::Embedding;
pub use query::{Naive, Query};
pub use std::io;

pub mod graph;
pub mod parsing;
pub mod query;
pub mod uwutree;
pub mod vec;

type NodeId = usize;
