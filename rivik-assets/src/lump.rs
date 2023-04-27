/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! A format for storing related assets in a single file

use byteorder::{LittleEndian, ReadBytesExt};
use fasthash::FastHasher;
use memmap2::{Mmap, MmapOptions};
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};
use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    io::{self, Cursor, Read, Seek},
    ops::Deref,
    path::Path,
    sync::Arc,
};

use crate::{get_cache, insert_cache};

#[derive(Snafu, Debug, Clone)]
pub enum ChunkFetchError {
    #[snafu(display("Hash function did not match hash function used to create lump file"))]
    WrongHasher { backtrace: Backtrace },

    #[snafu(display("Hash function was not recorded by this lump file"))]
    HasherUnspecifiedByFile {
        #[snafu(backtrace)]
        source: MissingChunkError,
    },

    #[snafu(display("Failed to find chunk with name {name}"))]
    NotFound {
        name: String,
        #[snafu(backtrace)]
        source: MissingChunkError,
    },
}

#[derive(Snafu, Debug, Clone)]
#[snafu(display("Chunk {id:X} not found in lump"))]
pub struct MissingChunkError {
    id: u32,
    backtrace: Backtrace,
}

struct Entry {
    id: u32,
    offset: u64,
    len: u64,
}

pub struct Lump {
    toc: Vec<Entry>,
    reader: Mmap,
}

pub struct LumpChunk {
    start: usize,
    end: usize,
    lump: Arc<Lump>,
}

impl Deref for LumpChunk {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.lump.reader[self.start..self.end]
    }
}

impl AsRef<[u8]> for LumpChunk {
    fn as_ref(&self) -> &[u8] {
        &*self
    }
}

impl Read for LumpChunk {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.end - self.start);
        let read_to = self.start + len;

        for (byte, out) in self.lump.reader[self.start..read_to]
            .iter()
            .zip(buf.iter_mut())
        {
            *out = *byte;
        }
        self.start = read_to;
        Ok(len)
    }
}

impl Lump {
    pub fn precache(path: &str) -> Arc<Lump> {
        let mut hash = DefaultHasher::new();
        path.hash(&mut hash);
        hash.write(b"lump");
        let hash = hash.finish();

        // check if we have already loaded this file
        match get_cache(hash) {
            Some(mmap) => mmap,
            None => {
                println!("Loading mmaped Lump file {path}");
                let file = File::open(path).unwrap();
                let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
                insert_cache(hash, Lump::new(mmap))
                    .downcast::<Lump>()
                    .unwrap()
            }
        }
    }

    fn new(map: Mmap) -> Lump {
        let mut reader: &[u8] = &*map;
        // parse the TOC from this reader
        let length = reader.read_u32::<LittleEndian>().unwrap();

        let mut toc = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let id = reader.read_u32::<LittleEndian>().unwrap();
            let offset = reader.read_u64::<LittleEndian>().unwrap();
            let len = reader.read_u32::<LittleEndian>().unwrap() as u64;
            toc.push(Entry { id, offset, len });
        }
        toc.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        Self { reader: map, toc }
    }

    pub fn chunk(&self, id: u32) -> Result<&[u8], MissingChunkError> {
        let index = self
            .toc
            .binary_search_by(|a| a.id.cmp(&id))
            .ok()
            .context(MissingChunkSnafu { id })?;
        let offset = self.toc[index].offset as usize;
        let len = self.toc[index].len as usize;

        Ok(&self.reader[offset..(offset + len)])
    }

    pub fn arc_chunk(self: Arc<Self>, id: u32) -> Result<LumpChunk, MissingChunkError> {
        let index = self
            .toc
            .binary_search_by(|a| a.id.cmp(&id))
            .ok()
            .context(MissingChunkSnafu { id })?;
        let offset = self.toc[index].offset as usize;
        let len = self.toc[index].len as usize;

        Ok(LumpChunk {
            start: offset,
            end: offset + len,
            lump: Arc::clone(&self),
        })
    }

    fn validate_hash<H: FastHasher>(&self) -> Result<(), ChunkFetchError> {
        let mut chunk = self.chunk(0).context(HasherUnspecifiedByFileSnafu)?;

        let mut hash = H::new();
        hash.write("LUMP".as_bytes());

        if hash.finish() == chunk.read_u64::<LittleEndian>().unwrap() {
            Ok(())
        } else {
            WrongHasherSnafu.fail()
        }
    }

    pub fn named_chunk<H: FastHasher>(&self, name: &str) -> Result<&[u8], ChunkFetchError> {
        self.validate_hash::<H>()?;

        let mut hash = H::new();
        hash.write(name.as_bytes());
        let id = hash.finish() as u32;
        self.chunk(id).context(NotFoundSnafu { name })
    }

    pub fn arc_named_chunk<H: FastHasher>(
        self: Arc<Self>,
        name: &str,
    ) -> Result<LumpChunk, ChunkFetchError> {
        self.validate_hash::<H>()?;

        let mut hash = H::new();
        hash.write(name.as_bytes());
        let id = hash.finish() as u32;
        self.arc_chunk(id).context(NotFoundSnafu { name })
    }
}
