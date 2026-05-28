# Veinq

A lightweight message broker written in Rust.

## Features

- Append only log storage with segment rolling
- O(1) reads via offset index
- Multiple topics and partitions
- Restart recovery
- Simple TCP protocol
- Rust client library

## Quick Start

### Start the server

```bash
cargo run -p veinq-server
```

### Connect from another Rust service

```toml
# Cargo.toml
[dependencies]
veinq-client = "0.1"
```

```rust
use veinq_client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("127.0.0.1:9092").await.unwrap();

    client.create_topic("orders", 3).await.unwrap();

    let offset = client.produce("orders", 0, b"hello").await.unwrap();
    let msg = client.consume("orders", 0, offset).await.unwrap();

    println!("{}", String::from_utf8_lossy(&msg));
}
```

## Configuration

Edit `config.toml`:

```toml
[broker]
host = "127.0.0.1"
port = 9092
data_dir = "logs"

[storage]
max_segment_size = 1073741824  # 1GB
```

## License

AGPL-3.0