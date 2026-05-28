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
use std::path::{Path, PathBuf};

use super::index::Index;

pub struct Segment {
    pub base_offset: u64,
    pub current_offset: u64,
    pub size_bytes: u64,
    file: File,
    path: PathBuf,
    index: Index,
}

impl Segment {
    pub fn new(dir: &Path, base_offset: u64) -> io::Result<Self> {
        let filename = format!("{:020}.log", base_offset);
        let path: PathBuf = dir.join(&filename);
        let file: File = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(&path)?;

        Ok(Self {
            base_offset,
            current_offset: base_offset,
            size_bytes: file.metadata()?.len(),
            file,
            path,
            index: Index::new(dir, base_offset)?,
        })
    }

    fn open(dir: &Path, base_offset: u64) -> io::Result<Self> {
        let filename = format!("{:020}.log", base_offset);
        let path: PathBuf = dir.join(&filename);
        let file: File = OpenOptions::new().read(true).append(true).open(&path)?;
        let current_offset: u64 = base_offset + Self::count_records(&path)?;
        Ok(Self {
            base_offset,
            current_offset: current_offset,
            size_bytes: file.metadata()?.len(),
            file,
            path,
            index: Index::new(dir, base_offset)?,
        })
    }

    fn count_records(file: &Path) -> io::Result<u64> {
        let mut count: u64 = 0u64;
        let mut file = File::open(file)?;
        let mut len_buf: [u8; 8] = [0u8; 8];
        loop {
            match file.read_exact(&mut len_buf) {
                Ok(_) => {
                    let len: u64 = u64::from_be_bytes(len_buf);
                    file.seek(SeekFrom::Current(len as i64))?;
                    count += 1;
                }
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            }
        }
        Ok(count)
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
        let offset: u64 = self.current_offset;
        let position: u64 = self.size_bytes;
        let len: u64 = data.len() as u64;
        println!(
            "Appending data of length {} at offset {} (position {}) base {}",
            len, offset, position, self.base_offset
        );
        let relative_offset: u64 = offset - self.base_offset;
        self.index.write(relative_offset, position)?;

        self.file.write_all(&len.to_be_bytes())?;
        self.file.write_all(data)?;
        self.current_offset += 1;
        self.size_bytes += 8 + len; // 8 bytes for length prefix
        Ok(offset)
    }

    pub fn read(&mut self, offset: u64) -> io::Result<Vec<u8>> {
        let relative_offset: u64 = offset - self.base_offset;
        let position: u64 = self.index.lookup(relative_offset)?;
        self.file.seek(SeekFrom::Start(position))?;
        let mut len_buf: [u8; 8] = [0u8; 8];
        self.file.read_exact(&mut len_buf)?;
        let len: usize = u64::from_be_bytes(len_buf) as usize;
        let mut data: Vec<u8> = vec![0u8; len];
        self.file.read_exact(&mut data)?;
        return Ok(data);
    }

    pub fn is_full(&self, max_segment_size: u64) -> bool {
        self.size_bytes >= max_segment_size
    }

    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.base_offset && offset < self.current_offset
    }
}

pub struct Log {
    dir: PathBuf,
    segments: Vec<Segment>,
    max_segment_size: u64,
}

impl Log {
    pub fn new(dir: &Path, max_segment_size: u64) -> io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let mut segments = Self::load_segments(dir)?;
        if segments.is_empty() {
            println!("No existing segments found, starting with a new log");
            segments.push(Segment::new(dir, 0)?);
        } else {
            println!("Recovered {} segments from disk", segments.len());
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            segments,
            max_segment_size,
        })
    }

    fn load_segments(dir: &Path) -> io::Result<Vec<Segment>> {
        let mut segments: Vec<Segment> = vec![];
        let mut base_offsets: Vec<u64> = vec![];
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".log") {
                let stem = name_str.trim_end_matches(".log");
                if let Ok(base_offset) = stem.parse::<u64>() {
                    base_offsets.push(base_offset);
                }
            }
        }
        base_offsets.sort();
        for base_offset in base_offsets {
            segments.push(Segment::open(dir, base_offset)?);
        }
        Ok(segments)
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
        let max_size = self.max_segment_size;
        if self.active_segment().is_full(max_size) {
            let new_base_offset: u64 = self.active_segment().current_offset;
            println!("Creating new segment with base offset {}", new_base_offset);
            self.segments
                .push(Segment::new(&self.dir, new_base_offset)?);
        }
        self.active_segment().append(data)
    }

    pub fn read(&mut self, offset: u64) -> io::Result<Vec<u8>> {
        let segment = self.find_segment(offset)?;
        segment.read(offset)
    }

    pub fn current_offset(&self) -> u64 {
        self.segments
            .last()
            .expect("There should always be at least one segment")
            .current_offset
    }

    pub fn active_segment(&mut self) -> &mut Segment {
        self.segments
            .last_mut()
            .expect("There should always be at least one segment")
    }

    pub fn find_segment(&mut self, offset: u64) -> io::Result<&mut Segment> {
        self.segments
            .iter_mut()
            .rev()
            .find(|s: &&mut Segment| s.contains(offset))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Offset not found in any segment")
            })
    }
}
