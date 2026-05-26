use super::partition::Partition;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub struct Topic {
    pub name: String,
    partitions: HashMap<u32, Partition>,
    base_dir: PathBuf,
}

impl Topic {
    pub fn new(base_dir: &Path, name: &str, num_partitions: u32) -> io::Result<Self> {
        let mut partitions = HashMap::new();
        for id in 0..num_partitions {
            let partition = Partition::new(base_dir, name, id)?;
            partitions.insert(id, partition);
        }
        Ok(Self {
            name: name.to_string(),
            partitions,
            base_dir: base_dir.to_path_buf(),
        })
    }

    pub fn append(&mut self, partition_id: u32, data: &[u8]) -> io::Result<u64> {
        let partition = self.partitions.get_mut(&partition_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Partition {} not found", partition_id),
            )
        })?;
        partition.append(data)
    }

    pub fn read(&mut self, partition_id: u32, offset: u64) -> io::Result<Vec<u8>> {
        let partition = self.partitions.get_mut(&partition_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Partition {} not found", partition_id),
            )
        })?;
        partition.read(offset)
    }

    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }
}
