use std::{fmt, io};

use crate::{traits::Key, PathBuf};

/// Errors produced by `FileBackedLfuCache`.
#[derive(Debug, thiserror::Error)]
pub enum Error<E>
where
    E: std::error::Error,
{
    /// Cannot initialise the given path as a backing directory.
    ///
    /// This can happen if the path does not resolve to a directory.
    Init(PathBuf),

    /// An item cannot be found with this key in cache.
    NotInCache(Box<dyn Key>),

    /// An item cannot be found with this key on disk.
    NotOnDisk(Box<dyn Key>),

    /// An item cannot be found with this key either in cache or on disk.
    NotFound(Box<dyn Key>),

    /// An error occurred while serialising/deserialising.
    ///
    /// The inner type is the user-defined associated type of `AsyncFileRepr`.
    Serde(E),

    /// An error occurred while performing IO.
    Io(io::Error),

    /// An item with this key is temporarily immutable due to outstanding references.
    ///
    /// This can happen if you are holding a reference elsewhere, or if this item
    /// is in the process of being flushed to disk.
    Immutable(Box<dyn Key>),
}
impl<E> fmt::Display for Error<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        let repr = match self {
            Init(path) => format!("Cannot initialise {path:?} as a backing directory."),
            NotInCache(key) => format!("Cannot find an item with key {key:?} in cache"),
            NotOnDisk(key) => format!("Cannot find an item with key {key:?} on disk"),
            NotFound(key) => {
                format!("Cannot find an item with key {key:?} either in cache or on disk")
            }
            Serde(error) => {
                format!("An error occurred during serialisation/deserialisation: {error}")
            }
            Io(error) => {
                format!("An error occurred while performing IO: {error}")
            }
            Immutable(key) => format!(
                "An item with key {key:?} is temporarily immutable due to outstanding references"
            ),
        };

        write!(f, "{repr}")
    }
}
impl<E> From<io::Error> for Error<E>
where
    E: std::error::Error,
{
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
