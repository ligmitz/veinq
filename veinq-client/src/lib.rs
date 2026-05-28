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

use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum VeinqError {
    TopicAlreadyExists,
    TopicNotFound,
    PartitionNotFound,
    IOError(io::Error),
    InvalidResponse,
}

impl From<io::Error> for VeinqError {
    fn from(err: io::Error) -> Self {
        VeinqError::IOError(err)
    }
}

pub type Result<T> = std::result::Result<T, VeinqError>;

impl std::fmt::Display for VeinqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VeinqError::TopicAlreadyExists => write!(f, "Topic already exists"),
            VeinqError::TopicNotFound => write!(f, "Topic not found"),
            VeinqError::PartitionNotFound => write!(f, "Partition not found"),
            VeinqError::IOError(e) => write!(f, "IO error: {}", e),
            VeinqError::InvalidResponse => write!(f, "Invalid response from server"),
        }
    }
}

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub async fn create_topic(&mut self, topic: &str, num_partitions: u32) -> Result<()> {
        let topic_bytes = format!("{}\0", topic).into_bytes();
        let partition_bytes = num_partitions.to_be_bytes();

        let mut payload = vec![2u8]; // 2 = CreateTopic
        payload.extend_from_slice(&topic_bytes);
        payload.extend_from_slice(&partition_bytes);

        self.send_frame(&payload).await?;

        let response = self.read_frame().await?;
        match response.first() {
            Some(0x00) => Ok(()),
            Some(0x01) => Err(VeinqError::TopicAlreadyExists),
            _ => Err(VeinqError::InvalidResponse),
        }
    }

    pub async fn produce(&mut self, topic: &str, partition: u32, data: &[u8]) -> Result<u64> {
        let topic = format!("{}\0", topic).into_bytes();
        let partition_bytes = partition.to_be_bytes();

        let mut payload = vec![0u8];
        payload.extend_from_slice(&topic);
        payload.extend_from_slice(&partition_bytes);
        payload.extend_from_slice(data);

        self.send_frame(&payload).await?;
        let response = self.read_frame().await?;
        if let Ok(msg) = std::str::from_utf8(&response) {
            if msg.starts_with("Error:") {
                if msg.contains("not found") {
                    return Err(VeinqError::TopicNotFound);
                } else {
                    return Err(VeinqError::InvalidResponse);
                }
            }
        }
        if response.len() != 8 {
            return Err(VeinqError::InvalidResponse);
        }
        let offset = u64::from_be_bytes(response[0..8].try_into().unwrap());
        Ok(offset)
    }

    pub async fn consume(&mut self, topic: &str, partition: u32, offset: u64) -> Result<Vec<u8>> {
        let topic = format!("{}\0", topic).into_bytes();
        let partition_bytes = partition.to_be_bytes();
        let offset_bytes = offset.to_be_bytes();

        let mut payload = vec![1u8];
        payload.extend_from_slice(&topic);
        payload.extend_from_slice(&partition_bytes);
        payload.extend_from_slice(&offset_bytes);

        self.send_frame(&payload).await?;
        let response = self.read_frame().await?;
        if let Ok(msg) = std::str::from_utf8(&response) {
            if msg.starts_with("Error:") {
                if msg.contains("not found") {
                    return Err(VeinqError::TopicNotFound);
                } else {
                    return Err(VeinqError::InvalidResponse);
                }
            }
        }
        Ok(response)
    }

    async fn send_frame(&mut self, payload: &[u8]) -> Result<()> {
        let len = (payload.len() as u32).to_be_bytes();
        self.stream.write_all(&len).await?;
        self.stream.write_all(payload).await?;
        Ok(())
    }

    async fn read_frame(&mut self) -> Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload).await?;
        Ok(payload)
    }
}
