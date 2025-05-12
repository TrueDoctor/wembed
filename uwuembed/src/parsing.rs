use core::panic;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::vec::DVec;

#[derive(Debug)]
pub struct Position {
    index: usize,
    weight: f64,
    coordinates: Vec<f64>,
}

#[derive(Debug)]
pub struct Iteration {
    number: usize,
    pub positions: Vec<Position>,
}

pub fn parse_positions_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Iteration>> {
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

            let num = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            current_iteration = Some(Iteration {
                number: num,
                positions: Vec::new(),
            });
        } else if line.contains("---") || line.split_whitespace().count() == 2 {
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

impl<const D: usize> From<&Iteration> for Vec<DVec<D>> {
    fn from(value: &Iteration) -> Self {
        if !value
            .positions
            .first()
            .is_some_and(|x| x.coordinates.len() == D)
        {
            panic!("Graph dimension from data file did not match the compiled graph dimension");
        }
        value
            .positions
            .iter()
            .map(|pos| DVec::from_fn(|i| pos.coordinates[i]))
            .collect()
    }
}
impl<const D: usize> From<Iteration> for Vec<DVec<D>> {
    fn from(value: Iteration) -> Self {
        (&value).into()
    }
}
