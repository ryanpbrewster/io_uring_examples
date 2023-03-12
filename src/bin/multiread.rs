use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    os::{fd::AsRawFd, unix::prelude::OsStrExt},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use hdrhistogram::{Histogram, SyncHistogram};
use io_uring::types;
use rand::{rngs::SmallRng, Rng, SeedableRng};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    path: PathBuf,

    #[arg(long, default_value_t = 2)]
    reads_per_iter: usize,

    #[arg(long, value_enum)]
    variant: Variant,

    #[arg(long, value_enum)]
    mode: Mode,

    #[arg(long, value_enum)]
    method: Method,
}

#[derive(Copy, Clone, ValueEnum)]
enum Variant {
    Pread,
    Preadv,
}

#[derive(Copy, Clone, ValueEnum)]
enum Mode {
    Cached,
    Direct,
}

#[derive(Copy, Clone, ValueEnum)]
enum Method {
    Blocking,
    Uring,
}

const BLOCK_WIDTH: u64 = 512;

fn main() {
    let args = Args::parse();

    let mut hist: SyncHistogram<u32> = SyncHistogram::from(Histogram::new(5).unwrap());
    let mut recorder = hist.recorder();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(1_000));
        hist.refresh();
        println!(
            "total={} p50={} p99={} p999={} p9999={}",
            hist.len(),
            hist.value_at_quantile(0.5),
            hist.value_at_quantile(0.99),
            hist.value_at_quantile(0.999),
            hist.value_at_quantile(0.9999),
        );
    });

    let mut prng = SmallRng::seed_from_u64(42);

    let md = std::fs::metadata(&args.path).unwrap();
    let num_keys = md.len() / BLOCK_WIDTH;
    println!(
        "file {:?} has {} bytes --> {} blocks",
        args.path,
        md.len(),
        num_keys
    );

    let cpath = CString::new(args.path.as_os_str().as_bytes()).unwrap();

    let flag = match args.mode {
        Mode::Cached => 0,
        Mode::Direct => libc::O_DIRECT,
    };
    let fd = unsafe { libc::open(cpath.as_ptr() as *const i8, flag, libc::O_RDONLY) };

    let mut keys = vec![0; args.reads_per_iter];
    let mut bufs: Vec<Vec<u8>> = vec![vec![0; BLOCK_WIDTH as usize]; args.reads_per_iter];

    match args.method {
        Method::Blocking => loop {
            let start = Instant::now();
            for i in 0..args.reads_per_iter {
                let offset = prng.gen_range(0..num_keys) * BLOCK_WIDTH;
                let r = unsafe {
                    libc::pread64(
                        fd,
                        bufs[i].as_mut_ptr() as *mut libc::c_void,
                        BLOCK_WIDTH as usize,
                        offset as i64,
                    )
                };
            }
            let elapsed = start.elapsed();
            recorder.record(elapsed.as_nanos() as u64).unwrap();
        },
        Method::Uring => {
            todo!()
        }
    }
}
