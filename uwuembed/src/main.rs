use uwuembed::{query::Update, *};

fn main() -> io::Result<()> {
    // Parse the bio-grid-fruitfly graph with 4 embedding dimensions
    // let graph = graph::Graph::parse_from_edge_list_file("bio-grid-fruitfly", 4, 4)?;
    let graph = graph::Graph::parse_from_edge_list_file("bio-grid-fruitfly", 8, 8)?;
    // let graph = graph::Graph::parse_from_edge_list_file("data/rel8/graph", 8, 8)?;
    // // Print the graph details
    // graph.save_weights_file("bio-grid-fruitfly-weights.txt");

    // Parse the positions file

    // let positions_path = "positions.log";
    let positions_path = "data/bio-grid-fruitfly/positions_8_8.log";
    // let positions_path = "data/rel8/positions_8_8.log";

    let iterations = parsing::parse_positions_file(positions_path)?;
    let embeddings: Vec<Embedding<8>> = iterations
        .iter()
        .map(|x| Embedding {
            positions: x.into(),
            graph: &graph,
        })
        .collect();

    // Print summary
    println!("Parsed {} iterations", iterations.len());
    let mut naive = Naive::new(&embeddings[0]);
    for (i, iter) in embeddings.iter().step_by(1).enumerate() {
        naive.update_positions(&iter.positions);
        let sum: usize = (0..(iter.positions.len()))
            .map(|i| naive.repelling_nodes(i).len())
            .sum::<usize>();
        // let sum_at: usize = (0..(iter.positions.len()))
        //     .map(|i| naive.attracting_nodes(i).len())
        //     .sum::<usize>();
        let sum_at = 0;
        println!(
            "Iteration {}: {} average nodes in radius, attracting: {}",
            i * 1,
            sum as f64 / iter.positions.len() as f64,
            sum_at as f64 / iter.positions.len() as f64,
        );
    }

    Ok(())
}
