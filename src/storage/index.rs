use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

const ENTRY_SIZE: usize = 16; // 8 bytes for offset and 8 bytes for length

pub struct Index {
    file: File,
}

impl Index {
    pub fn new(dir: &Path) -> io::Result<Self> {
        let index_path = dir.join("stratum.index");
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(index_path)?;
        Ok(Self { file })
    }

    pub fn write(&mut self, offset: u64, position: u64) -> io::Result<()> {
        self.file.write_all(&offset.to_be_bytes())?;
        self.file.write_all(&position.to_be_bytes())?;
        Ok(())
    }

    pub fn lookup(&mut self, offset: u64) -> io::Result<u64> {
        self.file
            .seek(SeekFrom::Start(offset * ENTRY_SIZE as u64))?;
        let mut read_buf = [0u8; 8];
        self.file.read_exact(&mut read_buf)?; // Read the offset (not used here, but we need to skip it)
        self.file.read_exact(&mut read_buf)?; // Read the position
        Ok(u64::from_be_bytes(read_buf))
    }
}
