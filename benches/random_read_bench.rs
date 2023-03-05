use criterion::{black_box, criterion_group, criterion_main, Criterion};
use io_uring_examples::{MmapDb, PreadDb, ReadDb};
use rand::Rng;

pub fn read_bench(c: &mut Criterion) {
    c.bench_function("read_cached", |b| {
        let mut r = ReadDb::open("data/1k.dat").unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key: u32 = prng.gen_range(0..1_000);
            let _ = black_box(r.get(key).unwrap());
        })
    });

    c.bench_function("pread_cached", |b| {
        let mut r = PreadDb::open("data/1k.dat").unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key: u32 = prng.gen_range(0..1_000);
            let _ = black_box(r.get(key).unwrap());
        })
    });

    c.bench_function("mmap_read", |b| {
        let mut r = MmapDb::open("data/1k.dat").unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key: u32 = prng.gen_range(0..1_000);
            let _ = black_box(r.get(key).unwrap());
        })
    });
}

criterion_group!(benches, read_bench);
criterion_main!(benches);
