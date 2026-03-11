use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use once_cell::sync::Lazy;

use crate::qpu::QPU;

/// Global LRU cache for shortest paths on a QPU graph.
///
/// Keyed by a stable hash of (modality, calibration timestamp, coupling map, start, end).
static PATH_CACHE: Lazy<Mutex<LruCache<u64, Arc<Vec<u32>>>>> = Lazy::new(|| {
    let cap = NonZeroUsize::new(4096).unwrap();
    Mutex::new(LruCache::new(cap))
});

fn hash_qpu(qpu: &QPU) -> u64 {
    let mut h = DefaultHasher::new();
    (qpu.modality as u8).hash(&mut h);
    qpu.calibration_ts.hash(&mut h);

    // Coupling map hashing: order-independent (bucket + sort).
    let mut edges: Vec<(u32, u32, u64)> = vec![];
    edges.reserve(qpu.coupling_map.len());
    for (a, neighs) in &qpu.coupling_map {
        for (b, w) in neighs {
            edges.push((*a, *b, w.to_bits()));
        }
    }
    edges.sort_unstable();
    edges.hash(&mut h);

    h.finish()
}

pub fn get_cached_path(qpu: &QPU, start: u32, end: u32) -> Option<Arc<Vec<u32>>> {
    let mut h = DefaultHasher::new();
    hash_qpu(qpu).hash(&mut h);
    start.hash(&mut h);
    end.hash(&mut h);
    let key = h.finish();

    PATH_CACHE
        .lock()
        .ok()
        .and_then(|mut cache| cache.get(&key).cloned())
}

pub fn put_cached_path(qpu: &QPU, start: u32, end: u32, path: Vec<u32>) {
    let mut h = DefaultHasher::new();
    hash_qpu(qpu).hash(&mut h);
    start.hash(&mut h);
    end.hash(&mut h);
    let key = h.finish();

    if let Ok(mut cache) = PATH_CACHE.lock() {
        cache.put(key, Arc::new(path));
    }
}
