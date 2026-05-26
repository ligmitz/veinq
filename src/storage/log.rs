use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::index::Index;

pub struct Log {
    file: File,
    // path: PathBuf,
    current_offset: u64,
    current_position: u64,
    index: Index,
}

impl Log {
    pub fn new(dir: &Path) -> io::Result<Self> {
        let path: PathBuf = dir.join("stratum.log");
        let file: File = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(&path)?;
        Ok(Self {
            file,
            //path,
            current_offset: 0,
            current_position: 0,
            index: Index::new(dir)?,
        })
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
        let offset: u64 = self.current_offset;
        let position: u64 = self.current_position;
        let len: u64 = data.len() as u64;

        self.index.write(offset, position)?;

        self.file.write_all(&len.to_be_bytes())?;
        self.file.write_all(data)?;
        self.current_offset += 1 as u64;
        self.current_position += 8 + len; // 8 bytes for length prefix
        Ok(offset)
    }

    pub fn read(&mut self, offset: u64) -> io::Result<Vec<u8>> {
        let position: u64 = self.index.lookup(offset)?;
        self.file.seek(SeekFrom::Start(position))?;
        let mut len_buf: [u8; 8] = [0u8; 8];
        self.file.read_exact(&mut len_buf)?;
        let len: usize = u64::from_be_bytes(len_buf) as usize;
        let mut data: Vec<u8> = vec![0u8; len];
        self.file.read_exact(&mut data)?;
        return Ok(data);
    }

    pub fn current_offset(&self) -> u64 {
        self.current_offset
    }
}
