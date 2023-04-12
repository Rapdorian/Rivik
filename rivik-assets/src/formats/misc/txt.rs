use std::io;

use snafu::{Backtrace, ResultExt, Snafu};

use crate::{formats::Format, Path, ReaderCreationError};
/// File format defintion for a text file
pub struct Txt;

#[derive(Snafu, Debug)]
pub enum ParseTxtError {
    #[snafu(display("Failed loading txt file"))]
    CreationError {
        #[snafu(backtrace)]
        source: ReaderCreationError,
    },
    #[snafu(display("Failed to read txt file"))]
    ReadError {
        source: io::Error,
        backtrace: Backtrace,
    },
}

impl Format for Txt {
    type Output = String;
    type Error = ParseTxtError;

    fn parse(&self, path: &Path) -> Result<String, ParseTxtError> {
        let mut buffer = String::new();
        path.reader()
            .context(CreationSnafu)?
            .read_to_string(&mut buffer)
            .context(ReadSnafu)?;
        Ok(buffer)
    }
}
