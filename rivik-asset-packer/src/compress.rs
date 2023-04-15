use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use brotli::enc::BrotliEncoderParams;
use serde::{Deserialize, Serialize};
use snafu::{Backtrace, ResultExt, Snafu};

#[derive(Snafu, Debug)]
pub enum CompressError {
    #[snafu(display("Io error on file {path}"))]
    IoError {
        path: String,
        source: io::Error,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionAlg {
    Brotli,
    Lzma,
    Deflate,
}

pub fn compress_file(
    out: impl AsRef<Path>,
    file: impl AsRef<Path>,
    alg: CompressionAlg,
) -> Result<(), CompressError> {
    // copy the file to disk while encrypting it
    let mut in_file = File::open(&file).context(IoSnafu {
        path: file.as_ref().display().to_string(),
    })?;

    let out_file = File::create(&out).context(IoSnafu {
        path: file.as_ref().display().to_string(),
    })?;

    let mut out: Box<dyn Write> = match alg {
        CompressionAlg::Brotli => Box::new(brotli::CompressorWriter::with_params(
            out_file,
            2048,
            &BrotliEncoderParams::default(),
        )),
        CompressionAlg::Lzma => Box::new(lzma::LzmaWriter::new_compressor(out_file, 5).unwrap()),
        CompressionAlg::Deflate => Box::new(flate2::write::DeflateEncoder::new(
            out_file,
            flate2::Compression::best(),
        )),
    };

    let mut buf = Vec::new();
    in_file.read_to_end(&mut buf).unwrap();
    out.write_all(&buf).unwrap();
    Ok(())
}
