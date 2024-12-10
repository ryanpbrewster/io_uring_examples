use std::{fs::File, path::PathBuf};

use byteorder::{LittleEndian, WriteBytesExt};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    output: PathBuf,

    #[arg(long)]
    count: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut fout = File::create(args.output)?;
    for i in 0..args.count {
        fout.write_u64::<LittleEndian>(i)?;
    }

    Ok(())
}
