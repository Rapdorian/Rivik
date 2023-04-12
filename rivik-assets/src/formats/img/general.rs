use std::io::BufReader;

use crate::{formats::Format, Path, ReaderCreationError};

use image::DynamicImage;
pub use image::ImageFormat;
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
}

impl Format for Img {
    type Output = DynamicImage;
    type Error = ImageParseError;
    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        let reader = BufReader::new(path.reader().context(CreationSnafu)?);

        Ok(image::load(reader, self.0).context(ImageSnafu)?)
    }
}
