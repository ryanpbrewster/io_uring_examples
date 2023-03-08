use std::{
    ffi::CString,
    fs::File,
    io::{Seek, SeekFrom},
    os::unix::prelude::{FileExt, OsStrExt},
    path::Path, sync::{Arc, Mutex},
};

use anyhow::anyhow;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use libc::c_void;
use memmap::Mmap;

const WIDTH: usize = std::mem::size_of::<u64>();

pub trait Db : Send + Sync {
    fn get(&self, key: u64) -> anyhow::Result<u64>;
}

pub struct ReadDb {
    underlying: Arc<Mutex<File>>,
}

impl ReadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let underlying = File::open(path)?;
        Ok(Self { underlying: Arc::new(Mutex::new(underlying)) })
    }
}
impl Db for ReadDb {
    fn get(&self, key: u64) -> anyhow::Result<u64> {
        let mut g = self.underlying.lock().unwrap();
        g.seek(SeekFrom::Start(WIDTH as u64 * key))?;
        Ok(g.read_u64::<LittleEndian>()?)
    }
}

pub struct DirectPreadDb {
    fd: i32,
}

const BLOCK_WIDTH: usize = 512;

// O_DIRECT will reject operations if the file is not a multiple of 512 bytes in
// size, or if the MEMORY passed in is not aligned to 512 byte chunks. This is
// one way to coerce the allocator into giving us aligned memory.
#[repr(align(4096))]
struct Aligned([u8; BLOCK_WIDTH]);

impl DirectPreadDb {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // I ran into issues where the path wasn't being properly null-terminated, resulting in file-not-found errors.
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        let fd = unsafe { libc::open(path.as_ptr() as *const i8, libc::O_DIRECT, libc::O_RDONLY) };
        if fd < 0 {
            return Err(anyhow!(std::io::Error::last_os_error()));
        }

        Ok(Self { fd })
    }
}
impl Db for DirectPreadDb {
    fn get(&self, key: u64) -> anyhow::Result<u64> {
        let offset = WIDTH as u64 * key as u64;
        let intra_block_offset = offset % BLOCK_WIDTH as u64;
        let block_offset = offset - intra_block_offset;

        let buf = Aligned([0; BLOCK_WIDTH]);
        let result = unsafe {
            libc::pread(
                self.fd,
                buf.0.as_ptr() as *mut c_void,
                BLOCK_WIDTH,
                block_offset as i64,
            )
        };
        if result < 0 {
            return Err(anyhow!(std::io::Error::last_os_error()));
        }
        Ok(LittleEndian::read_u64(
            &buf.0[intra_block_offset as usize..][..WIDTH],
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
}
impl Db for PreadDb {
    fn get(&self, key: u64) -> anyhow::Result<u64> {
        let mut buf = [0; WIDTH];
        self.underlying
            .read_at(&mut buf, WIDTH as u64 * key as u64)?;
        Ok(LittleEndian::read_u64(&buf))
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
}
impl Db for MmapDb {
    fn get(&self, key: u64) -> anyhow::Result<u64> {
        Ok(LittleEndian::read_u64(
            &self.buf[WIDTH * key as usize..][..WIDTH],
        ))
    }
}

pub struct TokioUringDb {
    underlying: tokio_uring::fs::File,
}

impl TokioUringDb {
    pub async fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        use tokio_uring::fs::File;
        let underlying = File::open(path).await?;
        Ok(Self {
            underlying: underlying,
        })
    }
    pub async fn get(&self, key: u64) -> anyhow::Result<u64> {
        let buf = vec![0; WIDTH];
        let (res, buf) = self
            .underlying
            .read_at(buf, WIDTH as u64 * key as u64)
            .await;
        let _n = res?;
        Ok(LittleEndian::read_u64(&buf))
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use byteorder::{LittleEndian, WriteBytesExt};
    use tempfile::NamedTempFile;

    use crate::{DirectPreadDb, TokioUringDb, Db};

    fn setup_dataset(num_entries: u64) -> anyhow::Result<NamedTempFile> {
        let mut named = tempfile::NamedTempFile::new()?;
        let fout = named.as_file_mut();
        for i in 0..num_entries {
            fout.write_u64::<LittleEndian>(i)?;
        }
        fout.flush()?;
        Ok(named)
    }

    #[test]
    fn direct_pread_smoke() -> anyhow::Result<()> {
        let dataset = setup_dataset(128)?;
        let r = DirectPreadDb::open(dataset.path())?;
        assert_eq!(r.get(0)?, 0);
        assert_eq!(r.get(127)?, 127);
        Ok(())
    }

    #[test]
    fn tokio_iouring_smoke() -> anyhow::Result<()> {
        let dataset = setup_dataset(128)?;
        tokio_uring::start(async {
            let r = TokioUringDb::open(dataset.path()).await?;
            assert_eq!(r.get(0).await?, 0);
            assert_eq!(r.get(127).await?, 127);
            Ok(())
        })
    }
}
