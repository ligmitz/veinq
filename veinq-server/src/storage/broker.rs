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
use super::topic::Topic;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;

pub struct Broker {
    topics: HashMap<String, Topic>,
    base_dir: String,
    max_segment_size: u64,
}

impl Broker {
    pub fn new(base_dir: &Path, max_segment_size: u64) -> io::Result<Self> {
        create_dir_all(base_dir)?;
        let mut broker = Self {
            topics: HashMap::new(),
            base_dir: base_dir.to_string_lossy().to_string(),
            max_segment_size,
        };
        broker.recover()?;
        Ok(broker)
    }

    fn recover(&mut self) -> io::Result<()> {
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Some((topic_name, partition_id)) = dir_name.rsplit_once('-') {
                    let partition_id: u32 = partition_id.parse().unwrap_or(0);
                    let topic = self
                        .topics
                        .entry(topic_name.to_string())
                        .or_insert_with(|| Topic::new_empty(topic_name));
                    topic.partitions.insert(
                        partition_id,
                        Partition::new(
                            &Path::new(&self.base_dir),
                            topic_name,
                            partition_id,
                            self.max_segment_size,
                        )?,
                    );
                }
            }
        }
        if self.topics.is_empty() {
            println!("No existing topics found, starting with an empty broker");
        } else {
            println!("Recovered topics: {:?}", self.topics.keys());
        }
        Ok(())
    }

    pub fn create_topic(&mut self, name: &str, num_partitions: u32) -> io::Result<()> {
        if self.topics.contains_key(name) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Topic {} already exists", name),
            ));
        }
        let topic = Topic::new(
            Path::new(&self.base_dir),
            name,
            num_partitions,
            self.max_segment_size,
        )?;
        self.topics.insert(name.to_string(), topic);
        Ok(())
    }

    pub fn append(&mut self, topic_name: &str, partition_id: u32, data: &[u8]) -> io::Result<u64> {
        let topic = self.topics.get_mut(topic_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Topic {} not found", topic_name),
            )
        })?;
        topic.append(partition_id, data)
    }

    pub fn read(
        &mut self,
        topic_name: &str,
        partition_id: u32,
        offset: u64,
    ) -> io::Result<Vec<u8>> {
        let topic = self.topics.get_mut(topic_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Topic {} not found", topic_name),
            )
        })?;
        topic.read(partition_id, offset)
    }
}
