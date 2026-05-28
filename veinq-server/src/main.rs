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

mod config;
mod server;
mod storage;
use config::Config;
use std::path::Path;
use std::sync::{Arc, Mutex};
use storage::Broker;

#[tokio::main]
async fn main() {
    let config = Config::from_file(Path::new("config.toml")).unwrap_or_else(|err| {
        eprintln!("Failed to read config file: {}, using default config", err);
        Config::default()
    });
    println!("Starting Veinq with config: {:?}", config);
    let broker = Broker::new(
        Path::new(&config.server.logs_dir),
        config.storage.max_segment_size,
    )
    .expect("Failed to initialize broker");
    let broker = Arc::new(Mutex::new(broker));
    server::run(broker, config.addr()).await;
}
