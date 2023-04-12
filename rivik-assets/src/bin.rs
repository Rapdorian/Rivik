//! A binary container format for storing arbitrary buffers
//!
//! The format consists of a series of chunks that contain 32bit type indicator and a binary
//! buffer.
//! Buffers are stored sequentially and have their length encoded in the header allowing for chunks
//! to be easily skipped.
//!
//! # Usage
//!
//! ```
//! // Writing
//! # use std::io::Cursor;
//! # use asset_inator::bin::{Chunk, BinWrite, BinRead};
//! # let mut file = Cursor::new(Vec::new());
//! file.write_chunk(Chunk::new(0xC0D3, vec![0xDA, 0x7A]));
//!
//! // Reading
//! while let Ok(chunk) = file.chunk() {
//!     let chunk = chunk.read().unwrap();
//!     println!("chunk: {}", chunk.kind());
//!     println!("\t{:?}", chunk.bytes());
//! }
//! ```

use std::io::{Cursor, Read, Result as IoResult, Seek, SeekFrom, Write};

use log::trace;

/// Metadata added to a chunk
#[derive(Debug, Clone, Copy)]
pub struct Header {
    id: u128,
    len: u32,
    ty: u32,
}

/// A Chunk of binary data with some metadata for storing and retrieving
#[derive(Debug)]
pub struct Chunk {
    header: Header,
    data: Cursor<Vec<u8>>,
}

impl Read for Chunk {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.data.read(buf)
    }
}

impl Seek for Chunk {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        self.data.seek(pos)
    }
}

/// A handle to an unread chunk
///
/// Call `read` to finalize the chunk and read its contents into memory
#[derive(Debug)]
pub struct ChunkHandle<'a, T: Read + Seek> {
    header: Header,
    read: bool,
    data: &'a mut T,
}

impl<'a, T: Read + Seek> Drop for ChunkHandle<'a, T> {
    fn drop(&mut self) {
        // if we drop the chunk handle without reading we need to seek to the next chunk
        if !self.read {
            self.data
                .seek(SeekFrom::Current(self.header.len as i64))
                .unwrap();
        }
    }
}

impl<'a, T: Read + Seek> ChunkHandle<'a, T> {
    pub fn read(mut self) -> IoResult<Chunk> {
        trace!(
            "Reading contents of {:X} buffer {:X}",
            self.header.ty,
            self.header.id
        );
        let mut data = vec![0; self.header.len as usize];
        self.data.read_exact(&mut data)?;
        trace!(
            "Finished reading contents of {:X}, found {:?}",
            self.header.id,
            data
        );
        self.read = true;

        Ok(Chunk {
            header: self.header,
            data: Cursor::new(data),
        })
    }

    pub fn kind(&self) -> u32 {
        self.header.ty
    }

    pub fn id(&self) -> u128 {
        self.header.id
    }
}

impl<T: Read + Seek> Read for ChunkHandle<'_, T> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.data.read(buf)
    }
}

impl Chunk {
    /// Creates a chunk with a specific type marker
    pub fn new(ty: u32, data: Vec<u8>) -> Chunk {
        Chunk {
            header: Header {
                id: rand::random(),
                len: data.len() as u32,
                ty,
            },
            data: Cursor::new(data),
        }
    }

    /// Gets the buffer associated with this chunk
    pub fn bytes(&self) -> &[u8] {
        self.data.get_ref()
    }

    /// Gets the uuid of this chunk
    pub fn id(&self) -> u128 {
        self.header.id
    }

    /// Gets the data type of this chunk
    pub fn kind(&self) -> u32 {
        self.header.ty
    }
}

/// The `BinWrite` trait allows writing a bin chunk to the end of a writer
pub trait BinWrite: Write {
    /// Writes a chunk to the end of a writer
    fn write_chunk(&mut self, chunk: Chunk) -> IoResult<()> {
        self.write_all(&chunk.header.id.to_le_bytes())?;
        self.write_all(&chunk.header.len.to_le_bytes())?;
        self.write_all(&chunk.header.ty.to_le_bytes())?;
        self.write_all(chunk.data.get_ref())?;
        Ok(())
    }
}

/// The `BinRead` trait allows reading chunks from a reader
///
/// Ideally you should read chunks sequentially from a file
/// ```
/// # use std::io::Cursor;
/// # let mut reader = Cursor::new(Vec::new());
/// # use asset_inator::bin::BinRead;
/// while let Ok(chunk) = reader.chunk() {
///     // process chunk
/// }
/// ```
///
/// Note that this method does not actually read the contents of the chunks buffer, instead
/// returning a handle to the buffer. Use `chunk.read()` to finish reading the chunk if you need
/// its contents.
pub trait BinRead: Read + Seek {
    fn chunk(&mut self) -> IoResult<ChunkHandle<'_, Self>>
    where
        Self: Sized,
    {
        // parse header
        let header = {
            // create buffer for header
            let id = &mut [0; 16];
            let len = &mut [0; 4];
            let ty = &mut [0; 4];

            // read header into buffers
            self.read_exact(id)?;
            self.read_exact(len)?;
            self.read_exact(ty)?;

            // turn buffers into a header
            Header {
                id: u128::from_le_bytes(*id),
                len: u32::from_le_bytes(*len),
                ty: u32::from_le_bytes(*ty),
            }
        };

        // Create chunk handle
        Ok(ChunkHandle {
            data: self,
            header,
            read: false,
        })
    }
}

impl<T: Write> BinWrite for T {}
impl<T: Read + Seek> BinRead for T {}
