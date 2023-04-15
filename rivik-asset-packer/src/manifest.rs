//! Parse a description of a LUMP file and create it

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

use brotli::enc::BrotliEncoderParams;
use serde::{Deserialize, Serialize};

use crate::{
    compress::CompressionAlg,
    pack::{pack, FastHash, PackError},
};

/// An entry in a LUMP file.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "EntryRepr")]
#[serde(into = "EntryRepr")]
pub struct Entry {
    path: String,
    name: Option<String>,
    compress: Option<CompressionAlg>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum EntryRepr {
    Path(String),
    Entry {
        path: String,
        compress: Option<CompressionAlg>,
        name: Option<String>,
    },
}

impl From<Entry> for EntryRepr {
    fn from(value: Entry) -> Self {
        if value.compress.is_none() && value.name.is_none() {
            EntryRepr::Path(value.path)
        } else {
            EntryRepr::Entry {
                path: value.path,
                compress: value.compress,
                name: value.name,
            }
        }
    }
}

impl From<EntryRepr> for Entry {
    fn from(value: EntryRepr) -> Self {
        match value {
            EntryRepr::Path(path) => Entry::new(path),
            EntryRepr::Entry {
                path,
                compress,
                name,
            } => Entry {
                path,
                name,
                compress,
            },
        }
    }
}

impl Entry {
    /// Create a new entry from a file path
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: None,
            compress: None,
        }
    }

    /// Compress the contents of this entry when it is written into the LUMP file
    pub fn compress(self, alg: CompressionAlg) -> Self {
        Self {
            compress: Some(alg),
            ..self
        }
    }

    /// Rename the ID used to lookup this entry in the LUMP file
    pub fn rename(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Get the ID used to lookup this entry
    pub fn name(&self) -> &str {
        match self.name.as_ref() {
            Some(name) => name.as_str(),
            None => self.path.as_str(),
        }
    }

    /// Get the file path the contents of this entry will be read from
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    /// Get a reader to the contents of this entry
    ///
    /// Note that this will apply any compression that has been setup for this entry
    pub fn reader(&self) -> io::Result<Box<dyn Read>> {
        let file = File::open(&self.path)?;
        Ok(match &self.compress {
            Some(alg) => match alg {
                CompressionAlg::Brotli => Box::new(brotli::CompressorReader::with_params(
                    file,
                    2048,
                    &BrotliEncoderParams::default(),
                )),
                CompressionAlg::Lzma => {
                    Box::new(lzma::LzmaReader::new_compressor(file, 5).unwrap())
                }
                CompressionAlg::Deflate => Box::new(flate2::read::DeflateEncoder::new(
                    file,
                    flate2::Compression::best(),
                )),
            },
            None => Box::new(file),
        })
    }
}

/// Describes the contents of a LUMP file
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(from = "ManifestRepr")]
#[serde(into = "ManifestRepr")]
pub struct ManifestBuilder {
    hash: FastHash,
    files: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ManifestRepr {
    hash: FastHash,
    files: HashMap<String, Entry>,
}

impl From<ManifestBuilder> for ManifestRepr {
    fn from(value: ManifestBuilder) -> Self {
        let mut entries = HashMap::new();
        for mut entry in value.files {
            let name = entry.name().to_string();
            entry.name = None;
            entries.insert(name, entry);
        }
        Self {
            hash: value.hash,
            files: entries,
        }
    }
}

impl From<ManifestRepr> for ManifestBuilder {
    fn from(value: ManifestRepr) -> Self {
        let mut entries = Vec::new();
        for (k, v) in value.files.into_iter() {
            if v.name.is_none() {
                entries.push(v.rename(k));
            } else {
                entries.push(v);
            }
        }
        Self {
            hash: value.hash,
            files: entries,
        }
    }
}

impl ManifestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the hash function used to compute lookup ids
    pub fn with_hash(self, hash: FastHash) -> Self {
        Self { hash, ..self }
    }

    /// Add an entry to this file
    pub fn push(mut self, entry: Entry) -> Self {
        self.files.push(entry);
        self
    }

    /// Write the finished LUMP file
    pub fn build(self, out: impl AsRef<Path>) -> Result<(), PackError> {
        pack(out, &self.files, &self.hash)
    }
}
