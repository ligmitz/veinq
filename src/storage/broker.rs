use super::topic::Topic;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;

pub struct Broker {
    topics: HashMap<String, Topic>,
    base_dir: String,
}

impl Broker {
    pub fn new(base_dir: &Path) -> io::Result<Self> {
        create_dir_all(base_dir)?;
        Ok(Self {
            topics: HashMap::new(),
            base_dir: base_dir.to_string_lossy().to_string(),
        })
    }

    pub fn create_topic(&mut self, name: &str, num_partitions: u32) -> io::Result<()> {
        if self.topics.contains_key(name) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Topic {} already exists", name),
            ));
        }
        let topic = Topic::new(Path::new(&self.base_dir), name, num_partitions)?;
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
