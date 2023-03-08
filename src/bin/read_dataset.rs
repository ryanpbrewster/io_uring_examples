use std::{path::PathBuf, time::{Instant, Duration}, sync::{Arc, Mutex}};

use clap::{Parser, ValueEnum};
use hdrhistogram::Histogram;
use io_uring_examples::{ReadDb, PreadDb, MmapDb, DirectPreadDb, Db};
use rand::{Rng, rngs::SmallRng, SeedableRng};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    max_key: u64,

    #[arg(long)]
    concurrency: usize,

    #[arg(long, value_enum)]
    variant: Variant,
}

#[derive(Copy, Clone, ValueEnum)]
enum Variant {
    Read,
    Pread,
    DirectPread,
    Mmap,
}

fn main() {
    let args = Args::parse();

    let hist: Arc<Mutex<Histogram<u32>>> = Arc::new(Mutex::new(Histogram::new(5).unwrap()));

    let r: Arc<dyn Db> = match args.variant {
        Variant::Read => Arc::new(ReadDb::open(args.input).unwrap()),
        Variant::Pread => Arc::new(PreadDb::open(args.input).unwrap()),
        Variant::DirectPread => Arc::new(DirectPreadDb::open(args.input).unwrap()),
        Variant::Mmap => Arc::new(MmapDb::open(args.input).unwrap()),
    };

    for i in 0 .. args.concurrency {
        let r = r.clone();
        let hist = hist.clone();
        std::thread::spawn(move || {
            let mut prng = SmallRng::seed_from_u64(i as u64);
            loop {
                let start = Instant::now();
                r.get(prng.gen_range(0 .. args.max_key)).unwrap();
                let elapsed = start.elapsed();
                hist.lock().unwrap().record(elapsed.as_nanos() as u64).unwrap();
            }
        });
    }

    loop {
        std::thread::sleep(Duration::from_millis(1_000));
        let hist = hist.lock().unwrap();
        println!(
            "p50={:.1} p99={:.1} p999={:.1} avg={:.1} total={:.3e}",
            1e-3 * hist.value_at_quantile(0.50) as f64,
            1e-3 * hist.value_at_quantile(0.99) as f64,
            1e-3 * hist.value_at_quantile(0.999) as f64,
            1e-3 * hist.mean(),
            hist.len() as f64,
        );
    }
}