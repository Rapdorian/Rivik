//! Tool for generating packed lump files from a series of game assets.

use std::{
    fs::{self, File},
    hash::Hasher,
    io::{self, BufWriter, Seek, Write},
    mem::size_of,
    path::Path,
};

use byteorder::{LittleEndian, WriteBytesExt};
use clap::ValueEnum;
use snafu::{Backtrace, ResultExt, Snafu};

#[derive(ValueEnum, Debug, Clone, Default)]
#[clap(rename_all = "snake_case")]
pub enum FastHash {
    City,
    Farm,
    Metro,
    Mum,
    Murmur,
    Murmur2,
    Murmur3,
    #[default]
    Sea,
    Spooky,
    T1ha,
    Xx,
}

impl FastHash {
    pub fn new(&self) -> Box<dyn Hasher> {
        use fasthash::FastHasher;
        match self {
            FastHash::City => Box::new(fasthash::CityHasher::new()),
            FastHash::Farm => Box::new(fasthash::FarmHasher::new()),
            FastHash::Metro => Box::new(fasthash::MetroHasher::new()),
            FastHash::Mum => Box::new(fasthash::MumHasher::new()),
            FastHash::Murmur => Box::new(fasthash::MurmurHasher::new()),
            FastHash::Murmur2 => Box::new(fasthash::Murmur2Hasher::new()),
            FastHash::Murmur3 => Box::new(fasthash::Murmur3Hasher::new()),
            FastHash::Sea => Box::new(fasthash::SeaHasher::new()),
            FastHash::Spooky => Box::new(fasthash::SpookyHasher::new()),
            FastHash::T1ha => Box::new(fasthash::T1haHasher::new()),
            FastHash::Xx => Box::new(fasthash::XXHasher::new()),
        }
    }
}

fn version(hash: &FastHash) -> Vec<u8> {
    let mut hasher = hash.new();
    hasher.write("LUMP".as_bytes());
    hasher.finish().to_le_bytes().to_vec()
}

#[derive(Snafu, Debug)]
pub enum PackError {
    #[snafu(display("IO error on file: {path}"))]
    IoError {
        path: String,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Error while writing data to output file"))]
    FailedToWriteData {
        source: io::Error,
        backtrace: Backtrace,
    },
}

pub fn pack(
    out: impl AsRef<Path>,
    files: &[impl AsRef<str>],
    hash: &FastHash,
) -> Result<(), PackError> {
    let mut contents = vec![(0, String::from("version"), version(hash))];

    // generate buffers
    for file in files {
        let file = file.as_ref();
        let mut hasher = hash.new();
        hasher.write(file.as_bytes());
        let id = hasher.finish() as u32;

        let bytes = fs::read(&file).context(IoSnafu { path: file })?;
        contents.push((id, file.to_string(), bytes));
    }

    // stitch buffers together
    contents.sort_unstable_by_key(|a| a.0);
    let mut file = BufWriter::new(File::create(&out).context(IoSnafu {
        path: out.as_ref().display().to_string(),
    })?);
    file.write_u32::<LittleEndian>(contents.len() as u32)
        .context(FailedToWriteDataSnafu)?;
    println!(
        "Writing {} buffers into file: {}",
        contents.len(),
        out.as_ref().display()
    );

    // write table of contents
    let toc_size = (contents.len() * (size_of::<u32>() + size_of::<u64>() + size_of::<u64>()))
        + size_of::<u32>();

    let mut offset = toc_size;
    for (id, _, buffer) in &contents {
        file.write_u32::<LittleEndian>(*id)
            .context(FailedToWriteDataSnafu)?;
        file.write_u64::<LittleEndian>(offset as u64)
            .context(FailedToWriteDataSnafu)?;
        file.write_u64::<LittleEndian>(buffer.len() as u64)
            .context(FailedToWriteDataSnafu)?;
        offset += buffer.len();
    }

    // write in buffers
    for (id, name, buffer) in contents {
        println!(
            "0x{id:X}({name}) at offset: 0x{:X}",
            file.stream_position().context(FailedToWriteDataSnafu)?
        );

        file.write_all(&buffer).context(FailedToWriteDataSnafu)?
    }
    Ok(())
}