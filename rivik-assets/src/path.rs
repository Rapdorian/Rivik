use std::{
    error::Error as StdError,
    fmt::Display,
    fs::File,
    io::{self, Read, Seek},
    path::PathBuf,
};

use snafu::{Backtrace, GenerateImplicitData, IntoError, Snafu};
use uriparse::{Scheme, URIReference};

use crate::bin::BinRead;

/// Path used to identify an asset
///
/// defaults to a file path if scheme is not specificed
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Path {
    /// Filesystem path
    /// Loads the asset from disk using the provided filepath
    File(PathBuf),
    /// Chunk
    /// Loads a chunk file from disk and optionally fetches a single chunk from that file
    /// # Example
    /// `bin:path/to/file.bin#CHUNKID`
    Chunk(PathBuf, Option<u128>),
}

#[derive(Snafu, Debug)]
pub enum PathParseError {
    #[snafu(display("{uri} is not a valid URI"))]
    NotAURI {
        uri: String,
        source: Box<dyn StdError + Send + Sync>,
        backtrace: Backtrace,
    },
    #[snafu(display("unsuppored uri scheme: {scheme}"))]
    UnsupportedScheme {
        scheme: String,
        backtrace: Backtrace,
    },
}

impl PathParseError {
    pub fn not_a_uri<E: StdError + 'static + Send + Sync>(uri: &str, err: E) -> Self {
        NotAURISnafu {
            uri: uri.to_owned(),
        }
        .into_error(Box::new(err))
    }

    pub fn unsupported_scheme(scheme: &str) -> Self {
        UnsupportedSchemeSnafu {
            scheme: scheme.to_owned(),
        }
        .build()
    }
}

impl TryFrom<&str> for Path {
    type Error = PathParseError;

    fn try_from(path: &str) -> Result<Self, PathParseError> {
        let not_a_uri = |e| PathParseError::not_a_uri(path, e);

        let uri = URIReference::try_from(path).map_err(not_a_uri)?;

        // fetch an normalize scheme
        let mut scheme = uri.scheme().unwrap_or(&Scheme::File).clone();
        scheme.normalize();

        let not_a_uri = |e| PathParseError::not_a_uri(path, e);
        match scheme.as_ref() {
            "file" => Ok(Path::File(uri.path().to_string().into())),
            "bin" => Ok(Path::Chunk(
                uri.path().to_string().into(),
                uri.fragment()
                    .map(|frag| u128::from_str_radix(frag, 16))
                    .transpose()
                    .map_err(not_a_uri)?,
            )),
            scheme => Err(PathParseError::unsupported_scheme(scheme)),
        }
    }
}

impl TryFrom<String> for Path {
    type Error = PathParseError;

    fn try_from(value: String) -> Result<Self, PathParseError> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&String> for Path {
    type Error = PathParseError;

    fn try_from(value: &String) -> Result<Self, PathParseError> {
        Self::try_from(value.as_str())
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::File(p) => match p.is_absolute() {
                true => write!(f, "file://{}", p.to_string_lossy()),
                false => write!(f, "file:{}", p.to_string_lossy()),
            },
            Path::Chunk(p, Some(id)) => match p.is_absolute() {
                true => write!(f, "bin://{}#{:X}", p.to_string_lossy(), id),
                false => write!(f, "bin:{}#{:X}", p.to_string_lossy(), id),
            },
            Path::Chunk(p, None) => match p.is_absolute() {
                true => write!(f, "bin://{}", p.to_string_lossy()),
                false => write!(f, "bin:{}", p.to_string_lossy()),
            },
        }
    }
}

#[derive(Snafu, Debug)]
pub enum ReaderCreationError {
    #[snafu(display("Failed to create reader from {path}"))]
    IOError {
        path: Path,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not create reader from {path} due to missing chunk {id}"))]
    MissingChunk {
        path: Path,
        id: u128,
        backtrace: Backtrace,
    },
    #[snafu(display("Unsupported path format"))]
    UnsupportedPath { path: Path, backtrace: Backtrace },
}

impl ReaderCreationError {
    pub fn io_error(path: &Path, err: io::Error) -> Self {
        IOSnafu { path: path.clone() }.into_error(err)
    }

    pub fn missing_chunk(path: &Path, id: u128) -> Self {
        MissingChunkSnafu {
            path: path.clone(),
            id,
        }
        .build()
    }

    pub fn unsupported_path(path: &Path) -> Self {
        UnsupportedPathSnafu { path: path.clone() }.build()
    }
}

pub(crate) trait AssetReader: Read + Seek {}
impl<T: Read + Seek> AssetReader for T {}

impl Path {
    pub(crate) fn reader(&self) -> Result<Box<dyn AssetReader>, ReaderCreationError> {
        let io_error = |e| ReaderCreationError::io_error(self, e);
        match self {
            Path::File(path) => Ok(Box::new(File::open(path).map_err(io_error)?)),
            Path::Chunk(path, Some(id)) => {
                // find chunk in file
                let mut file = File::open(path).map_err(io_error)?;
                while let Ok(chunk) = file.chunk() {
                    if chunk.id() == *id {
                        return Ok(Box::new(chunk.read().map_err(io_error)?));
                    }
                }
                Err(ReaderCreationError::missing_chunk(self, *id))
            }
            _ => Err(ReaderCreationError::unsupported_path(self)),
        }
    }
}
