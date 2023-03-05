use std::{
    ffi::CString,
    fs::File,
    io::{Seek, SeekFrom},
    os::unix::prelude::{FileExt, OsStrExt},
    path::Path,
};

use anyhow::anyhow;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use libc::c_void;
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

pub struct DirectPreadDb {
    fd: i32,
}

const BLOCK_WIDTH: usize = 512;
const MASK: usize = BLOCK_WIDTH - 1;

impl DirectPreadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // I hate this code and don't really know how much of it is necessary.
        // I ran into issues where the path wasn't being properly null-terminated, resulting in file-not-found errors.
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        let fd = unsafe { libc::open(path.as_ptr() as *const i8, libc::O_DIRECT, libc::O_RDONLY) };
        if fd < 0 {
            return Err(anyhow!(std::io::Error::last_os_error()));
        }

        Ok(Self { fd })
    }
    pub fn get(&self, key: u32) -> anyhow::Result<u32> {
        let offset = WIDTH as u64 * key as u64;
        let intra_block_offset = offset % BLOCK_WIDTH as u64;
        let block_offset = offset - intra_block_offset;

        let buf = [0; 2 * BLOCK_WIDTH];
        let alignment_offset = BLOCK_WIDTH - (buf.as_ptr() as usize & MASK);
        let result = unsafe {
            libc::pread(
                self.fd,
                (buf.as_ptr() as usize + alignment_offset) as *mut c_void,
                BLOCK_WIDTH,
                block_offset as i64,
            )
        };
        if result < 0 {
            return Err(anyhow!(std::io::Error::last_os_error()));
        }
        Ok(LittleEndian::read_u32(
            &buf[alignment_offset + intra_block_offset as usize..][..WIDTH],
        ))
    }
}
impl Drop for DirectPreadDb {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
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
    pub fn get(&self, key: u32) -> anyhow::Result<u32> {
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
    pub fn get(&self, key: u32) -> anyhow::Result<u32> {
        Ok(LittleEndian::read_u32(
            &self.buf[WIDTH * key as usize..][..WIDTH],
        ))
    }
}
