//! Tomb asset manager
//!
//! This asset manager caches assets as they are loaded.
//! The cache is accessible from all threads but only grabs a thread safe handle once per thread.
//! Caches the threadsafe handle and creates thread local handles to the thread safe handle to
//! minimize atomic stores.
//!
//! # Usage
//!
//! ```
//! # use asset_inator::{load, formats::misc::Txt};
//! load("file:Cargo.toml", Txt).unwrap();
//! ```
//!
//! Some asset types will also need some parameters to be given to the parser. For example
//! ```
//! # use asset_inator::{load, formats::img::{ImageFormat, Img}};
//! load("file:examples/face.jpg", Img(ImageFormat::Jpeg)).unwrap();
//! ```

pub mod bin;
//pub mod handle;
mod mgr;
mod path;
use std::{backtrace::Backtrace, error::Error as StdError, fmt};

pub use formats::Format;
pub use mgr::*;
pub use path::*;

/// File formats implementations
pub mod formats {
    use std::{any::Any, fmt};

    use crate::Path;

    pub trait Asset: Any + Sync {}

    /// The `format` trait provides an interface for parsing a block of data into an asset
    pub trait Format {
        type Output;
        type Error;

        /// Parse a reader into some kind of asset
        fn parse(&self, r: &Path) -> Result<Self::Output, Self::Error>;
    }

    pub trait FormatError: snafu::ErrorCompat + std::error::Error + snafu::AsErrorSource {}
    impl<E> FormatError for E where E: snafu::ErrorCompat + std::error::Error + snafu::AsErrorSource {}

    pub struct ConcreteFormatError {
        pub(crate) inner: Box<dyn FormatError + Send + Sync>,
    }

    impl snafu::ErrorCompat for ConcreteFormatError {
        fn backtrace(&self) -> Option<&snafu::Backtrace> {
            self.inner.backtrace()
        }
    }

    // impl snafu::AsErrorSource for ConcreteFormatError {
    //     fn as_error_source(&self) -> &(dyn snafu::Error + 'static) {
    //         self.inner.as_error_source()
    //     }
    // }

    impl std::error::Error for ConcreteFormatError {
        fn source(&self) -> Option<&(dyn snafu::Error + 'static)> {
            self.inner.source()
        }
    }

    impl fmt::Display for ConcreteFormatError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl fmt::Debug for ConcreteFormatError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.inner.fmt(f)
        }
    }

    pub mod mesh;
    pub mod misc;
    pub mod img {
        mod general;
        pub use general::*;
    }
}
