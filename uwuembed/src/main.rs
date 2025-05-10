use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct Position {
    index: usize,
    weight: f64,
    coordinates: Vec<f64>,
}

#[derive(Debug)]
enum Query {
    Nearest {
        k: u32,
        point: Vec<f64>,
    },
    Sphere {
        radius: f64,
        min_corners: Vec<f64>,
        max_corners: Vec<f64>,
        points: Vec<f64>,
    },
    Box {
        min_corners: Vec<f64>,
        max_corners: Vec<f64>,
    },
}

#[derive(Debug)]
struct Iteration {
    number: usize,
    positions: Vec<Position>,
    queries: Vec<Query>,
}

fn parse_positions_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Iteration>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut iterations = Vec::new();
    let mut current_iteration = None;
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        if line.starts_with("ITERATION") {
            if let Some(iter) = current_iteration.take() {
                iterations.push(iter);
            }
            
            let num = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
                
            current_iteration = Some(Iteration {
                number: num,
                positions: Vec::new(),
                queries: Vec::new(),
            });
        } else if line.contains("---") {
            // End of positions block
            continue;
        } else if let Some(ref mut iter) = current_iteration {
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.len() >= 2 && parts[1].parse::<f64>().is_ok() {
                // This is a position line
                let index = parts[0].parse::<usize>().unwrap_or(0);
                let weight = parts[1].parse::<f64>().unwrap_or(0.0);
                let dim = parts[2].parse::<usize>().unwrap_or(0);
                
                let mut coordinates = Vec::with_capacity(dim);
                for i in 0..dim {
                    if let Some(val) = parts.get(3 + i) {
                        if let Ok(coord) = val.parse::<f64>() {
                            coordinates.push(coord);
                        }
                    }
                }
                
                iter.positions.push(Position {
                    index,
                    weight,
                    coordinates,
                });
            } else if parts.len() >= 2 {
                // This is a header line with positions count and dimension
                // Already handled by subsequent position lines
            }
        }
    }
    
    // Don't forget the last iteration
    if let Some(iter) = current_iteration {
        iterations.push(iter);
    }
    
    Ok(iterations)
}

fn parse_queries_file<P: AsRef<Path>>(path: P, iterations: &mut [Iteration]) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut current_iteration_idx = 0;
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        if line.starts_with("ITERATION") {
            let num = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
                
            current_iteration_idx = iterations.iter()
                .position(|iter| iter.number == num)
                .unwrap_or(0);
        } else if line.contains("---") {
            // Separator, skip
            continue;
        } else if line.starts_with("NEAREST") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let k = parts[1].parse::<u32>().unwrap_or(0);
            let dim = parts[2].parse::<usize>().unwrap_or(0);
            
            let mut point = Vec::with_capacity(dim);
            for i in 0..dim {
                if let Some(val) = parts.get(3 + i) {
                    if let Ok(coord) = val.parse::<f64>() {
                        point.push(coord);
                    }
                }
            }
            
            if let Some(iter) = iterations.get_mut(current_iteration_idx) {
                iter.queries.push(Query::Nearest { k, point });
            }
        } else if line.starts_with("SPHERE") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let radius = parts[1].parse::<f64>().unwrap_or(0.0);
            let dim = parts[2].parse::<usize>().unwrap_or(0);
            
            let mut min_corners = Vec::with_capacity(dim);
            let mut max_corners = Vec::with_capacity(dim);
            let mut points = Vec::with_capacity(dim);
            
            for i in 0..dim {
                let base_idx = 3 + i * 3;
                
                if let (Some(min), Some(max), Some(point)) = (
                    parts.get(base_idx).and_then(|v| v.parse::<f64>().ok()),
                    parts.get(base_idx + 1).and_then(|v| v.parse::<f64>().ok()),
                    parts.get(base_idx + 2).and_then(|v| v.parse::<f64>().ok())
                ) {
                    min_corners.push(min);
                    max_corners.push(max);
                    points.push(point);
                }
            }
            
            if let Some(iter) = iterations.get_mut(current_iteration_idx) {
                iter.queries.push(Query::Sphere {
                    radius,
                    min_corners,
                    max_corners,
                    points,
                });
            }
        } else if line.starts_with("BOX") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let dim = parts[1].parse::<usize>().unwrap_or(0);
            
            let mut min_corners = Vec::with_capacity(dim);
            let mut max_corners = Vec::with_capacity(dim);
            
            for i in 0..dim {
                let base_idx = 2 + i * 2;
                
                if let (Some(min), Some(max)) = (
                    parts.get(base_idx).and_then(|v| v.parse::<f64>().ok()),
                    parts.get(base_idx + 1).and_then(|v| v.parse::<f64>().ok())
                ) {
                    min_corners.push(min);
                    max_corners.push(max);
                }
            }
            
            if let Some(iter) = iterations.get_mut(current_iteration_idx) {
                iter.queries.push(Query::Box {
                    min_corners,
                    max_corners,
                });
            }
        }
    }
    
    Ok(())
}

fn main() -> io::Result<()> {
    let positions_path = "positions.log";
    let queries_path = "queries.log";
    
    let mut iterations = parse_positions_file(positions_path)?;
    parse_queries_file(queries_path, &mut iterations)?;
    
    // Print summary
    println!("Parsed {} iterations", iterations.len());
    for iter in &iterations {
        println!(
            "Iteration {}: {} positions, {} queries", 
            iter.number, 
            iter.positions.len(), 
            iter.queries.len()
        );
    }
    
    Ok(())
}
