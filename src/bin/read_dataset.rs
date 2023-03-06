use std::{path::PathBuf, time::Instant};

use clap::{Parser, ValueEnum};
use hdrhistogram::Histogram;
use io_uring_examples::{TokioUringDb, ReadDb, PreadDb, MmapDb, DirectPreadDb};
use rand::Rng;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    size: u32,

    #[arg(long)]
    iterations: u32,

    #[arg(long, value_enum)]
    variant: Variant,
}

#[derive(Copy, Clone, ValueEnum)]
enum Variant {
    Read,
    Pread,
    DirectPread,
    Mmap,
    TokioUring,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let hist = match args.variant {
        Variant::Read => bench_read(args),
        Variant::Pread => bench_pread(args),
        Variant::DirectPread => bench_direct_pread(args),
        Variant::Mmap => bench_mmap(args),
        Variant::TokioUring => bench_tokio_uring(args),
    };
    
    

    println!(
        "total={} p50={} p99={} avg={}",
        hist.len(),
        hist.value_at_quantile(0.50),
        hist.value_at_quantile(0.99),
        hist.mean()
    );
    Ok(())
}

fn bench_read(args: Args) -> Histogram<u32> {
    let mut prng = rand::thread_rng();
    let mut hist: Histogram<u32> = Histogram::new(5).unwrap();

    let mut r = ReadDb::open(args.input).unwrap();
    for _ in 0..args.iterations {
        let key = prng.gen_range(0..args.size);
        let start = Instant::now();
        let value = r.get(key).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(value, key);
        hist.record(elapsed.as_nanos() as u64).unwrap();
    }
    hist
}

fn bench_pread(args: Args) -> Histogram<u32> {
    let mut prng = rand::thread_rng();
    let mut hist: Histogram<u32> = Histogram::new(5).unwrap();

    let r = PreadDb::open(args.input).unwrap();
    for _ in 0..args.iterations {
        let key = prng.gen_range(0..args.size);
        let start = Instant::now();
        let value = r.get(key).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(value, key);
        hist.record(elapsed.as_nanos() as u64).unwrap();
    }
    hist
}

fn bench_mmap(args: Args) -> Histogram<u32> {
    let mut prng = rand::thread_rng();
    let mut hist: Histogram<u32> = Histogram::new(5).unwrap();

    let r = MmapDb::open(args.input).unwrap();
    for _ in 0..args.iterations {
        let key = prng.gen_range(0..args.size);
        let start = Instant::now();
        let value = r.get(key).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(value, key);
        hist.record(elapsed.as_nanos() as u64).unwrap();
    }
    hist
}

fn bench_direct_pread(args: Args) -> Histogram<u32> {
    let mut prng = rand::thread_rng();
    let mut hist: Histogram<u32> = Histogram::new(5).unwrap();

    let r = DirectPreadDb::open(args.input).unwrap();
    for _ in 0..args.iterations {
        let key = prng.gen_range(0..args.size);
        let start = Instant::now();
        let value = r.get(key).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(value, key);
        hist.record(elapsed.as_nanos() as u64).unwrap();
    }
    hist
}

fn bench_tokio_uring(args: Args) -> Histogram<u32> {
    let mut prng = rand::thread_rng();
    let mut hist: Histogram<u32> = Histogram::new(5).unwrap();

    tokio_uring::start(async move {
        let r = TokioUringDb::open(args.input).await.unwrap();
        for _ in 0..args.iterations {
            let key = prng.gen_range(0..args.size);
            let start = Instant::now();
            let value = r.get(key).await.unwrap();
            let elapsed = start.elapsed();
            assert_eq!(value, key);
            hist.record(elapsed.as_nanos() as u64).unwrap();
        }
        hist
    })
}