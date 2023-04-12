use std::{
    any::{type_name, Any, TypeId},
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    mem,
    rc::{self, Rc},
    sync::{self, Arc, RwLock},
};

use log::trace;
use once_cell::{sync::Lazy, unsync};
use snafu::{Backtrace, GenerateImplicitData, Snafu};

use crate::{
    formats::{ConcreteFormatError, Format, FormatError},
    path::Path,
    PathParseError,
};

static ASSET_CACHE: Lazy<RwLock<HashMap<u64, sync::Weak<dyn Any + Sync + Send>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

thread_local! {
    #[allow(clippy::type_complexity)]
    static THREAD_ASSET_CACHE: unsync::Lazy<RefCell<HashMap<u64,rc::Weak<Arc<dyn Any + Sync + Send>>>>>  = unsync::Lazy::new(|| RefCell::new(HashMap::new()));
}

#[derive(Debug, Snafu)]
pub enum AssetLoadError {
    #[snafu(display("Failed to parse path"))]
    PathError {
        #[snafu(backtrace)]
        source: PathParseError,
    },
    #[snafu(display("Failed to downcast asset at {path} to {ty}"))]
    DowncastError {
        path: Path,
        ty: &'static str,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed to load asset {path} from format"))]
    FormatError {
        path: Path,
        #[snafu(backtrace)]
        source: ConcreteFormatError,
    },
}

impl AssetLoadError {
    pub fn path_error(source: PathParseError) -> Self {
        AssetLoadError::PathError { source }
    }

    pub fn downcast_error(path: Path, ty: &'static str) -> Self {
        AssetLoadError::DowncastError {
            path,
            ty,
            backtrace: Backtrace::generate(),
        }
    }

    pub fn format_error<E: FormatError + 'static + Send + Sync>(path: Path, source: E) -> Self {
        AssetLoadError::FormatError {
            path,
            source: ConcreteFormatError {
                inner: Box::new(source),
            },
        }
    }
}

/// Load an asset from `path` it will load the asset using the file format specified by `format`
///
/// It will check a cache of previously loaded assets before loading the asset and if the asset
/// has not been cached it will cache the asset
pub fn load<F, P>(path: P, format: F) -> Result<Rc<Arc<F::Output>>, AssetLoadError>
where
    F: Format + Any,
    F::Output: Any + Send + Sync,
    F::Error: FormatError + Send + Sync,
    P: TryInto<Path, Error = PathParseError>,
{
    let path = path.try_into().map_err(|e| AssetLoadError::path_error(e))?;
    // generate a hash of the path and format
    let mut hash = DefaultHasher::new();
    path.hash(&mut hash);
    format.type_id().hash(&mut hash);
    let hash = hash.finish();

    let asset = THREAD_ASSET_CACHE.with(|cache| -> Result<_, AssetLoadError> {
        // check thread-local cache
        let key = cache.borrow().get(&hash).map(rc::Weak::clone);
        if let Some(asset) = key.and_then(|a| a.upgrade()) {
            return Ok(asset);
        }
        // check global cache
        let asset = Rc::new(load_cache(hash, path.clone(), format)?);
        cache.borrow_mut().insert(hash, Rc::downgrade(&asset));
        Ok(asset)
    })?;
    // this is some cursed shit
    // We need to manually implement downcast
    // first check if the types are the same
    if Arc::as_ref(&asset).type_id() != TypeId::of::<F::Output>() {
        return Err(AssetLoadError::downcast_error(
            path,
            type_name::<F::Output>(),
        ));
    }

    // I'm 80% sure this is sound
    let typed =
        unsafe { mem::transmute::<Rc<Arc<dyn Any + Send + Sync>>, Rc<Arc<F::Output>>>(asset) };
    Ok(typed)
}

/// Attempt to load an asset from the global cache
fn load_cache<F>(
    hash: u64,
    path: Path,
    format: F,
) -> Result<Arc<dyn Any + Send + Sync>, AssetLoadError>
where
    F: Format,
    F::Output: Any + Send + Sync,
    F::Error: FormatError + 'static + Send + Sync,
{
    if let Some(asset) = ASSET_CACHE
        .read()
        .unwrap()
        .get(&hash)
        .and_then(|a| a.upgrade())
    {
        trace!("found asset {} in global cache", path);
        return Ok(asset);
    }
    Ok(insert_cache(
        hash,
        load_asset(path.clone(), format).map_err(|e| AssetLoadError::format_error(path, e))?,
    ))
}

/// Add an asset to the global cache
fn insert_cache<A: Any + Send + Sync>(hash: u64, asset: A) -> Arc<dyn Any + Send + Sync> {
    let asset: Arc<dyn Any + Send + Sync> = Arc::new(asset);
    ASSET_CACHE
        .write()
        .unwrap()
        .insert(hash, Arc::downgrade(&asset));
    asset
}

/// Load an asset
/// doesn't handle the asset cache
fn load_asset<F>(path: Path, format: F) -> Result<F::Output, F::Error>
where
    F: Format,
{
    format.parse(&path)
}
