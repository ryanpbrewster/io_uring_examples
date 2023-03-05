use std::{
    fs::File,
    io::{Seek, SeekFrom},
    os::unix::prelude::FileExt,
    path::Path,
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use memmap::Mmap;

const WIDTH: usize = std::mem::size_of::<u32>();

pub struct ReadDb {
    underlying: File,
}

impl ReadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let underlying = File::open(path)?;
        Ok(Self { underlying })
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        self.underlying
            .seek(SeekFrom::Start(WIDTH as u64 * key as u64))?;
        Ok(self.underlying.read_u32::<LittleEndian>()?)
    }
}

pub struct PreadDb {
    underlying: File,
}

impl PreadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let underlying = File::open(path)?;
        Ok(Self { underlying })
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        let mut buf = [0; WIDTH];
        self.underlying
            .read_at(&mut buf, WIDTH as u64 * key as u64)?;
        Ok(LittleEndian::read_u32(&buf))
    }
}

pub struct MmapDb {
    _underlying: File,
    buf: Mmap,
}

impl MmapDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let underlying = File::open(path)?;
        let buf = unsafe { memmap::Mmap::map(&underlying) }?;
        Ok(Self {
            _underlying: underlying,
            buf,
        })
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        Ok(LittleEndian::read_u32(
            &self.buf[WIDTH * key as usize..][..WIDTH],
        ))
    }
}
