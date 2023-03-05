use std::fs::File;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use io_uring_examples::{MmapDb, ReadDb};
use rand::Rng;

pub fn read_bench(c: &mut Criterion) {
    c.bench_function("sync_read_cached", |b| {
        let file = File::open("data/1k.dat").unwrap();
        let mut r = ReadDb::new(file);
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key: u32 = prng.gen_range(0..1_000);
            let _ = black_box(r.get(key).unwrap());
        })
    });

    c.bench_function("mmap_read", |b| {
        let file = File::open("data/1k.dat").unwrap();
        let mut r = MmapDb::new(file).unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key: u32 = prng.gen_range(0..1_000);
            let _ = black_box(r.get(key).unwrap());
        })
    });
}

criterion_group!(benches, read_bench);
criterion_main!(benches);
