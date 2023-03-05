use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    os::unix::prelude::{FileExt, OpenOptionsExt, OsStrExt},
    path::Path,
};

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

pub struct DirectReadDb {
    fd: i32,
}


const BLOCK_WIDTH: usize = 512;
const MASK: usize = !(BLOCK_WIDTH - 1);
#[repr(C, align(512))]
struct MyBuf([u8; 512]);

impl DirectReadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref().as_os_str();
        println!("open({:?}, {}, {})", path, libc::O_DIRECT, libc::O_RDONLY);
        let fd = unsafe { libc::open(path.as_bytes().as_ptr() as * const i8, libc::O_DIRECT, libc::O_RDONLY) };
        println!("fd = {}", fd);
        Ok(Self { fd })
    }
    pub fn get(&mut self, key: u32) -> anyhow::Result<u32> {
        let offset = WIDTH as u64 * key as u64;
        let intra_block_offset = offset % BLOCK_WIDTH as u64;
        let block_offset = offset - intra_block_offset;
        let mut buf = [0; 2 * BLOCK_WIDTH];
        let mut ptr = buf.as_mut_ptr() as usize & MASK;
        println!("reading {} bytes (starting at {}), looking for offset={}", BLOCK_WIDTH, block_offset, offset);
        println!("pread({}, {:?}, {}, {})", self.fd, ptr, BLOCK_WIDTH, block_offset);
        let result = unsafe { libc::read(self.fd, ptr as *mut c_void, BLOCK_WIDTH) };
        println!("read, got result = {}", result);
        println!("{:?}", buf);
        if result < 0 {
            println!("last error = {:?}", std::io::Error::last_os_error());
        }
        Ok(LittleEndian::read_u32(&buf[intra_block_offset as usize..][..WIDTH]))
    }
}
impl Drop for DirectReadDb {
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
