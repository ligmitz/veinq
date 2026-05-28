// Copyright (C) 2026 Gaurav Pandey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::partition::Partition;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub struct Topic {
    pub name: String,
    pub partitions: HashMap<u32, Partition>,
    base_dir: PathBuf,
}

impl Topic {
    pub fn new(
        base_dir: &Path,
        name: &str,
        num_partitions: u32,
        max_segment_size: u64,
    ) -> io::Result<Self> {
        let mut partitions = HashMap::new();
        for id in 0..num_partitions {
            let partition = Partition::new(base_dir, name, id, max_segment_size)?;
            partitions.insert(id, partition);
        }
        Ok(Self {
            name: name.to_string(),
            partitions,
            base_dir: base_dir.to_path_buf(),
        })
    }

    pub fn new_empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            partitions: HashMap::new(),
            base_dir: PathBuf::new(),
        }
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
