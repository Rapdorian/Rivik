/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io;

use snafu::{Backtrace, ResultExt, Snafu};

use crate::{formats::Format, Path, ReaderCreationError};

/// File format defintion for a byte buffer
pub struct Bin;

#[derive(Snafu, Debug)]
pub enum ParseBinError {
    #[snafu(display("Failed loading bin file"))]
    CreationError {
        #[snafu(backtrace)]
        source: ReaderCreationError,
    },
    #[snafu(display("Failed to read bin file"))]
    ReadError {
        source: io::Error,
        backtrace: Backtrace,
    },
}

impl Format for Bin {
    type Output = Vec<u8>;
    type Error = ParseBinError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        let mut buffer = Vec::new();
        path.reader()
            .context(CreationSnafu)?
            .read_to_end(&mut buffer)
            .context(ReadSnafu)?;
        Ok(buffer)
    }
}
