use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use ordered_float::OrderedFloat;

use crate::cache::{get_cached_path, put_cached_path};

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Modality {
    Superconducting = 0,
    IonTrap = 1,
    NeutralAtom = 2,
}

#[derive(Debug)]
pub struct QPU {
    pub coupling_map: HashMap<u32, Vec<(u32, f64)>>,
    pub modality: Modality,
    pub positions: HashMap<u32, (f64, f64)>,
    pub calibration_ts: String,
}

impl QPU {
    pub fn new(modality: Modality, calibration_ts: String) -> Self {
        Self {
            coupling_map: HashMap::with_capacity(64),
            modality,
            positions: HashMap::with_capacity(64),
            calibration_ts,
        }
    }

    pub fn add_coupling(&mut self, a: u32, b: u32, err: f64) {
        self.coupling_map.entry(a).or_default().push((b, err));
    }

    pub fn add_position(&mut self, q: u32, x: f64, y: f64) {
        self.positions.insert(q, (x, y));
    }

    /// Rigorous lowest-obstruction path (BMS sector shift) for Superconducting only.
    /// Proof: standard Dijkstra with decrease-key via lazy heap + visited guard.
    /// Returns empty if unreachable.
    pub fn find_optimal_path(&self, start: u32, end: u32) -> Vec<u32> {
        if self.modality != Modality::Superconducting || start == end {
            return vec![start];
        }

        if let Some(path) = get_cached_path(self, start, end) {
            return (*path).clone();
        }

        let mut dist: HashMap<u32, f64> = HashMap::with_capacity(self.coupling_map.len());
        let mut prev: HashMap<u32, u32> = HashMap::with_capacity(self.coupling_map.len());
        let mut heap: BinaryHeap<Reverse<(OrderedFloat<f64>, u32)>> = BinaryHeap::new();
        let mut visited: HashSet<u32> = HashSet::with_capacity(self.coupling_map.len());

        dist.insert(start, 0.0);
        heap.push(Reverse((OrderedFloat(0.0), start)));

        while let Some(Reverse((cost, u))) = heap.pop() {
            if visited.contains(&u) {
                continue;
            }
            if cost.0 > *dist.get(&u).unwrap_or(&f64::INFINITY) {
                continue;
            }
            visited.insert(u);

            if u == end {
                break;
            }

            if let Some(neighs) = self.coupling_map.get(&u) {
                for &(v, weight) in neighs {
                    let next_cost = cost.0 + weight;
                    if next_cost < *dist.get(&v).unwrap_or(&f64::INFINITY) {
                        dist.insert(v, next_cost);
                        prev.insert(v, u);
                        heap.push(Reverse((OrderedFloat(next_cost), v)));
                    }
                }
            }
        }

        if !dist.contains_key(&end) {
            return vec![];
        }

        // Reconstruct (guaranteed shortest)
        let mut path = vec![];
        let mut cur = end;
        while cur != start {
            path.push(cur);
            cur = *prev.get(&cur).unwrap();
        }
        path.push(start);
        path.reverse();

        put_cached_path(self, start, end, path.clone());
        path
    }

    /// Hardware-invariant gate cost – used by QuantumQuantizer.
    pub fn gate_cost(&self, phys1: u32, phys2: u32) -> f64 {
        if phys1 == phys2 {
            return 0.0;
        }
        match self.modality {
            Modality::Superconducting => {
                let path = self.find_optimal_path(phys1, phys2);
                if path.len() < 2 {
                    return f64::INFINITY;
                }
                let mut cost = 0.0;
                for i in 0..path.len() - 1 {
                    if let Some(neighs) = self.coupling_map.get(&path[i]) {
                        if let Some((_, err)) = neighs.iter().find(|&&(nb, _)| nb == path[i + 1]) {
                            cost += err;
                        }
                    }
                }
                cost
            }
            Modality::IonTrap => 0.0012, // 2026 state-of-the-art
            Modality::NeutralAtom => {
                let d = if let (Some(p1), Some(p2)) =
                    (self.positions.get(&phys1), self.positions.get(&phys2))
                {
                    (p1.0 - p2.0).hypot(p1.1 - p2.1)
                } else {
                    10.0
                };
                0.004 + 0.018 * d
            }
        }
    }
}
