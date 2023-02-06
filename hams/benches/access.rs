use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::{
    iter,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

struct CheckMe {
    latest: Instant,
}

/// Create an alive check that takes a margin and fails when the time has not been kept up to date within the margin
impl CheckMe {
    pub fn new() -> Self {
        Self {
            latest: Instant::now(),
        }
    }

    /// Update the latest time record
    pub fn kick(&mut self) {
        self.latest = Instant::now();
    }

    pub fn get(&self) -> Instant {
        self.latest
    }
}

fn access_benchmark(c: &mut Criterion) {
    let mut access_group = c.benchmark_group("Access");

    access_group.bench_function("mutex write", |b| {
        let alive = Arc::new(Mutex::new(CheckMe::new()));

        b.iter(|| black_box(&alive).lock().unwrap().kick());
    });

    access_group.bench_function("mutex read", |b| {
        let alive = Arc::new(Mutex::new(CheckMe::new()));

        b.iter(|| black_box(&alive).lock().unwrap().get());
    });

    access_group.bench_function("rwlock write", |b| {
        let alive = Arc::new(RwLock::new(CheckMe::new()));

        b.iter(|| black_box(&alive).write().unwrap().kick());
    });

    access_group.bench_function("rwlock read with write", |b| {
        let alive = Arc::new(RwLock::new(CheckMe::new()));

        b.iter(|| black_box(&alive).write().unwrap().get());
    });

    access_group.bench_function("rwlock read", |b| {
        let alive = Arc::new(RwLock::new(CheckMe::new()));

        b.iter(|| black_box(&alive).read().unwrap().get());
    });

    access_group.finish();
}

criterion_group!(benches, access_benchmark);
criterion_main!(benches);
