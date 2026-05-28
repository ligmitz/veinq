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

use crate::storage::Broker;
use std::str;
use std::sync::{Arc, Mutex};
use std::vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn run(broker: Arc<Mutex<Broker>>, addr: String) {
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    println!("Veinq running on {}", addr);
    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        println!("Accepted connection from {}", addr);
        let broker_copy = Arc::clone(&broker);
        tokio::spawn(async move {
            handle_client(socket, broker_copy).await;
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream, broker: Arc<Mutex<Broker>>) {
    loop {
        let mut buf = [0u8; 4];
        if socket.read_exact(&mut buf).await.is_err() {
            println!("Client disconnected");
            return;
        }
        let len = u32::from_be_bytes(buf) as usize;
        let mut dataframe = vec![0u8; len];
        if socket.read_exact(&mut dataframe).await.is_err() {
            println!("Client disconnected");
            return;
        }
        let request_type: u8 = dataframe[0];
        let payload: &[u8] = &dataframe[1..];
        let response = match request_type {
            0 => handle_produce(payload, &broker),
            1 => handle_consume(payload, &broker),
            2 => handle_create_topic(payload, &broker),
            _ => {
                println!("Unknown request type: {}", request_type);
                b"Unknown request type".to_vec()
            }
        };
        let response_len = (response.len() as u32).to_be_bytes();
        if socket.write_all(&response_len).await.is_err() {
            println!("Client disconnected");
            return;
        }
        if socket.write_all(&response).await.is_err() {
            println!("Client disconnected");
            return;
        }
    }
}

fn handle_produce(payload: &[u8], broker: &Arc<Mutex<Broker>>) -> Vec<u8> {
    let null_pos = match payload.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => {
            println!("Invalid produce request: missing null separator");
            return b"Invalid produce request".to_vec();
        }
    };
    let topic = match str::from_utf8(&payload[..null_pos]) {
        Ok(s) => s,
        Err(_) => {
            println!("Invalid produce request: topic is not valid UTF-8");
            return b"Invalid produce request".to_vec();
        }
    };
    let rest = &payload[null_pos + 1..];
    let partition = if rest.len() < 4 {
        println!("Invalid produce request: missing partition");
        return b"Invalid produce request".to_vec();
    } else {
        let partition_bytes: [u8; 4] = rest[..4].try_into().unwrap();
        u32::from_be_bytes(partition_bytes)
    };
    let data = &rest[4..];
    let mut broker = broker.lock().unwrap();
    match broker.append(topic, partition, data) {
        Ok(offset) => {
            println!(
                "Produced to topic '{}', partition {}, offset {}",
                topic, partition, offset
            );
            offset.to_be_bytes().to_vec()
        }
        Err(e) => format!("Error: {}", e).into_bytes(),
    }
}

fn handle_consume(payload: &[u8], broker: &Arc<Mutex<Broker>>) -> Vec<u8> {
    let null_pos = match payload.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => {
            println!("Invalid consume request: missing null separator");
            return b"Invalid consume request".to_vec();
        }
    };
    let topic = match str::from_utf8(&payload[..null_pos]) {
        Ok(s) => s,
        Err(_) => {
            println!("Invalid consume request: topic is not valid UTF-8");
            return b"Invalid consume request".to_vec();
        }
    };
    let rest = &payload[null_pos + 1..];
    if rest.len() < 12 {
        println!("Invalid consume request: missing partition or offset");
        return b"Invalid consume request".to_vec();
    }
    let partition_bytes: [u8; 4] = rest[..4].try_into().unwrap();
    let partition = u32::from_be_bytes(partition_bytes);
    let offset_bytes: [u8; 8] = rest[4..12].try_into().unwrap();
    let offset = u64::from_be_bytes(offset_bytes);
    let mut broker = broker.lock().unwrap();
    match broker.read(topic, partition, offset) {
        Ok(data) => {
            println!(
                "Consumed from topic '{}', partition {}, offset {}",
                topic, partition, offset
            );
            data
        }
        Err(e) => format!("Error: {}", e).into_bytes(),
    }
}

fn handle_create_topic(payload: &[u8], broker: &Arc<Mutex<Broker>>) -> Vec<u8> {
    let null_pos = match payload.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => return vec![0x02],
    };
    let topic = match str::from_utf8(&payload[..null_pos]) {
        Ok(s) => s,
        Err(_) => return vec![0x02],
    };
    let rest = &payload[null_pos + 1..];
    if rest.len() < 4 {
        return vec![0x02];
    }
    let num_partitions: u32 = u32::from_be_bytes(rest[..4].try_into().unwrap());
    let mut broker = broker.lock().unwrap();
    match broker.create_topic(topic, num_partitions) {
        Ok(_) => {
            println!(
                "Created topic '{}' with '{}' partitions",
                topic, num_partitions
            );
            vec![0x00]
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                println!("Topic '{}' already exists", topic);
                vec![0x01]
            } else {
                println!("Error creating topic '{}': {}", topic, e);
                vec![0x02]
            }
        }
    }
}
