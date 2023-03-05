use criterion::{black_box, criterion_group, criterion_main, Criterion};
use io_uring_examples::{DirectPreadDb, MmapDb, PreadDb, ReadDb};
use rand::Rng;

pub fn read_bench(c: &mut Criterion) {
    for (name, size) in [("1k", 1 << 10), ("1m", 1 << 20), ("1g", 1 << 30)] {
        let path = format!("data/{}.dat", name);
        c.bench_function(&format!("read_cached_{}", name), |b| {
            let mut r = ReadDb::open(&path).unwrap();
            let mut prng = rand::thread_rng();
            b.iter(|| {
                let key: u32 = prng.gen_range(0..size);
                let _ = black_box(r.get(key).unwrap());
            })
        });

        c.bench_function(&format!("pread_cached_{}", name), |b| {
            let r = PreadDb::open(&path).unwrap();
            let mut prng = rand::thread_rng();
            b.iter(|| {
                let key: u32 = prng.gen_range(0..size);
                let _ = black_box(r.get(key).unwrap());
            })
        });

        c.bench_function(&format!("pread_direct_{}", name), |b| {
            let r = DirectPreadDb::open(&path).unwrap();
            let mut prng = rand::thread_rng();
            b.iter(|| {
                let key: u32 = prng.gen_range(0..size);
                let _ = black_box(r.get(key).unwrap());
            })
        });

        c.bench_function(&format!("mmap_read_{}", name), |b| {
            let r = MmapDb::open(&path).unwrap();
            let mut prng = rand::thread_rng();
            b.iter(|| {
                let key: u32 = prng.gen_range(0..size);
                let _ = black_box(r.get(key).unwrap());
            })
        });
    }
}

criterion_group!(benches, read_bench);
criterion_main!(benches);
