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
    pub fn new(base_dir: &Path, topic: &str, id: u32, max_segment_size: u64) -> io::Result<Self> {
        let dir: PathBuf = base_dir.join(format!("{}-{}", topic, id));
        create_dir_all(&dir)?;
        let log: Log = Log::new(&dir, max_segment_size)?;
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
