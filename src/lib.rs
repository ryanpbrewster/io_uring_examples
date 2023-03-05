use std::{
    fs::File,
    io::{Seek, SeekFrom},
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use memmap::Mmap;

pub struct ReadDb {
    underlying: File,
}

impl ReadDb {
    pub fn new(file: File) -> Self {
        Self { underlying: file }
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        self.underlying.seek(SeekFrom::Start(key as u64))?;
        Ok(self.underlying.read_u32::<LittleEndian>()?)
    }
}

pub struct MmapDb {
    _underlying: File,
    buf: Mmap,
}

const WIDTH: usize = std::mem::size_of::<u32>();
impl MmapDb {
    pub fn new(file: File) -> anyhow::Result<Self> {
        let buf = unsafe { memmap::Mmap::map(&file) }?;
        Ok(Self {
            _underlying: file,
            buf,
        })
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        Ok(LittleEndian::read_u32(
            &self.buf[WIDTH * key as usize..][..WIDTH],
        ))
    }
}
