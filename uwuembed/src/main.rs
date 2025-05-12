use graph::Embedding;
use std::io;

mod graph;
mod parsing;
mod query;
mod vec;

fn main() -> io::Result<()> {
    // Parse the bio-grid-fruitfly graph with 4 embedding dimensions
    let graph = graph::Graph::parse_from_edge_list_file("bio-grid-fruitfly")?;
    // Print the graph details
    graph.save_weights_file("bio-grid-fruitfly-weights.txt");

    // Parse the positions file

    let positions_path = "positions.log";

    let iterations = parsing::parse_positions_file(positions_path)?;
    let embeddings: Vec<Embedding<4>> = iterations
        .iter()
        .map(|x| Embedding {
            positions: x.into(),
            graph: &graph,
        })
        .collect();

    // Print summary
    println!("Parsed {} iterations", iterations.len());
    for (i, iter) in embeddings.iter().enumerate() {
        println!("Iteration {}: {} positions", i, iter.positions.len(),);
    }

    Ok(())
}
