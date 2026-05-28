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

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

const ENTRY_SIZE: usize = 16; // 8 bytes for offset and 8 bytes for length

pub struct Index {
    file: File,
}

impl Index {
    pub fn new(dir: &Path, base_offset: u64) -> io::Result<Self> {
        let filename = format!("{:020}.index", base_offset);
        let index_path = dir.join(&filename);
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(index_path)?;
        Ok(Self { file })
    }

    pub fn write(&mut self, offset: u64, position: u64) -> io::Result<()> {
        self.file.write_all(&offset.to_be_bytes())?;
        self.file.write_all(&position.to_be_bytes())?;
        Ok(())
    }

    pub fn lookup(&mut self, offset: u64) -> io::Result<u64> {
        self.file
            .seek(SeekFrom::Start(offset * ENTRY_SIZE as u64))?;
        let mut read_buf = [0u8; 8];
        self.file.read_exact(&mut read_buf)?; // Read the offset (not used here, but we need to skip it)
        self.file.read_exact(&mut read_buf)?; // Read the position
        Ok(u64::from_be_bytes(read_buf))
    }
}
