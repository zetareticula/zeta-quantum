//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

// Qubit and QPU structures for pathfinding

/// Represents a physical qubit in the quantum processor.
pub struct QubitNode {
    pub id: u32, // Unique identifier for the qubit
    pub error_rate: f64, // The 1-qubit gate decoherence
}

pub struct QPU {
    // Adjacency list: Mapping Qubit A to (Qubit B, CNOT Error Rate)
    pub coupling_map: HashMap<u32, Vec<(u32, f64)>>,
}

impl QPU {
    /// Find the 'Path of Least Obstruction' (BMS sector shift)
    /// Returns the sequence of qubits [start, ..., end] that minimizes
    /// the cumulative noise (sum of CNOT error rates along the edges).
    /// Uses a modified Dijkstra's algorithm on the noise-weighted directed graph.
    /// If no path exists, returns an empty Vec.
    pub fn find_optimal_path(&self, start: u32, end: u32) -> Vec<u32> {
        if start == end {
            return vec![start];
        }

        // Distance to each node (cumulative noise)
        let mut dist: HashMap<u32, f64> = HashMap::new();
        // Predecessor for path reconstruction
        let mut prev: HashMap<u32, u32> = HashMap::new();
        // Min-heap: (cumulative_noise, qubit_id)
        let mut heap: BinaryHeap<Reverse<(f64, u32)>> = BinaryHeap::new();

        dist.insert(start, 0.0);
        heap.push(Reverse((0.0, start)));

        while let Some(Reverse((cost, u))) = heap.pop() {
            // Skip outdated entries (a better path was already found)
            if cost > *dist.get(&u).unwrap_or(&f64::INFINITY) {
                continue;
            }

            // Explore neighbors
            if let Some(neighbors) = self.coupling_map.get(&u) {
                for &(v, weight) in neighbors {
                    let next_cost = cost + weight;

                    let current_best = dist.get(&v).copied().unwrap_or(f64::INFINITY);
                    if next_cost < current_best {
                        dist.insert(v, next_cost);
                        prev.insert(v, u);
                        heap.push(Reverse((next_cost, v)));
                    }
                }
            }
        }

        // No path found
        if !dist.contains_key(&end) {
            return Vec::new();
        }

        // Reconstruct path by backtracking predecessors
        let mut path = Vec::new();
        let mut current = end;
        while current != start {
            path.push(current);
            if let Some(&predecessor) = prev.get(&current) {
                current = predecessor;
            } else {
                // Disconnected (should not happen after dist check)
                return Vec::new();
            }
        }
        path.push(start);
        path.reverse();

        path
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_optimal_path() {
        let mut coupling_map = HashMap::new();
        coupling_map.insert(0, vec![(1, 0.01), (2, 0.02)]);
        coupling_map.insert(1, vec![(2, 0.01)]);
        coupling_map.insert(2, vec![]);

        let qpu = QPU { coupling_map };

        let path = qpu.find_optimal_path(0, 2);
        assert_eq!(path, vec![0, 1, 2]);
    }
}


