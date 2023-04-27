/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io::{self, BufReader, Cursor};

use crate::{formats::Format, Path, ReaderCreationError};

pub use image::ImageFormat;
use image::{io::Reader, DynamicImage};
use snafu::{Backtrace, ResultExt, Snafu};

/// Wrapper around using the `image` crate to parse images
pub struct Img(pub ImageFormat);

#[derive(Snafu, Debug)]
pub enum ImageParseError {
    #[snafu(display("Failed to read image"))]
    CreationError {
        #[snafu(backtrace)]
        source: ReaderCreationError,
    },
    #[snafu(display("Failed to parse image"))]
    ImageError {
        source: image::ImageError,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed to read buffer"))]
    ReadError {
        source: io::Error,
        backtrace: Backtrace,
    },
}

impl Format for Img {
    type Output = DynamicImage;
    type Error = ImageParseError;
    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        let mut buf = Vec::new();
        path.reader()
            .context(CreationSnafu)?
            .read_to_end(&mut buf)
            .context(ReadSnafu)?;
        Ok(Reader::new(Cursor::new(buf)).decode().context(ImageSnafu)?)
    }
}
