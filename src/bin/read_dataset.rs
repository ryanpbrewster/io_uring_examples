use std::path::PathBuf;

use clap::Parser;
use io_uring_examples::ReadDb;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    count: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut r = ReadDb::open(args.input)?;
    let mut total = 0;
    for i in 0..args.count {
        total += r.get(i)?;
    }
    println!("total = {}", total);

    Ok(())
}
