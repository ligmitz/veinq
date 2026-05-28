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

use veinq_client::{Client, VeinqError};

#[tokio::main]
async fn main() {
    let mut client = Client::connect("127.0.0.1:9092")
        .await
        .expect("failed to connect");

    match client.create_topic("orders", 3).await {
        Ok(_) => println!("topic created"),
        Err(VeinqError::TopicAlreadyExists) => println!("topic already exists — continuing"),
        Err(e) => panic!("failed to create topic: {}", e),
    }

    let offset = client
        .produce("orders", 0, b"hello from rust client")
        .await
        .expect("failed to produce");
    println!("produced at offset {}", offset);

    let msg = client
        .consume("orders", 0, offset)
        .await
        .expect("failed to fetch");
    println!("fetched: {}", String::from_utf8_lossy(&msg));

    match client.consume("unknown", 0, 0).await {
        Err(VeinqError::TopicNotFound) => println!("correctly got TopicNotFound error"),
        _ => println!("unexpected result"),
    }
}
