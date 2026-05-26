use super::log::Log;
use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};

pub struct Partition {
    pub topic: String,
    pub id: u32,
    log: Log,
}

impl Partition {
    pub fn new(base_dir: &Path, topic: &str, id: u32) -> io::Result<Self> {
        let dir: PathBuf = base_dir.join(format!("{}-{}", topic, id));
        create_dir_all(&dir)?;
        let log: Log = Log::new(&dir)?;
        Ok(Self {
            topic: topic.to_string(),
            id,
            log,
        })
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
        self.log.append(data)
    }

    pub fn read(&mut self, offset: u64) -> io::Result<Vec<u8>> {
        self.log.read(offset)
    }
}
