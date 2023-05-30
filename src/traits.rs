use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncRead, AsyncWrite},
};

use crate::{Error, Path};

/// A datatype that can be used as the access key for cached items.
///
/// Note that this datatype when converted into a path, should not contain values
/// that can be misinterpreted by the OS (e.g. path separators). I recommend
/// UUIDv4 for most use cases.
pub trait Key
where
    Self: Debug + Send + Sync + 'static,
{
    /// Generate a new, unique key.
    fn new() -> Self
    where
        Self: Sized;

    /// Convert this key to a filename used for flushing to disk.
    fn as_filename(&self) -> String;
}
#[cfg(feature = "uuid-as-key")]
impl Key for uuid::Uuid {
    fn new() -> Self {
        uuid::Uuid::new_v4()
    }

    fn as_filename(&self) -> String {
        self.to_string()
    }
}

/// A data structure with a file representation which can be loaded from
/// and flushed to disk asynchronously.
#[async_trait]
pub trait AsyncFileRepr
where
    Self: Sized,
{
    type Err: std::error::Error;

    /// Load (deserialise) the data structure into memory asynchronously.
    ///
    /// If you wish to perform non-trivial computation/conversion in this function,
    /// you should spawn a blocking task with your async runtime.
    async fn load<R>(reader: R) -> Result<Self, Self::Err>
    where
        R: Send + Unpin + AsyncRead;

    /// Flush (serialise) the data structure from memory asynchronously.
    ///
    /// If you wish to perform non-trivial computation/conversion in this function,
    /// you should spawn a blocking task with your async runtime.
    async fn flush<W>(self: &Arc<Self>, writer: W) -> Result<(), Self::Err>
    where
        W: Send + Unpin + AsyncWrite;

    /// Load (deserialise) the data structure from disk.
    async fn load_from_disk(
        path: impl AsRef<Path> + Send + Sync,
    ) -> Result<Self, Error<Self::Err>> {
        let file = OpenOptions::new().read(true).open(path.as_ref()).await?;
        let data = Self::load(file).await.map_err(Error::Serde)?;
        Ok(data)
    }

    /// Flush (serialise) the data structure to disk.
    async fn flush_to_disk(
        self: &Arc<Self>,
        path: impl AsRef<Path> + Send + Sync,
    ) -> Result<(), Error<Self::Err>>
    where
        Self: Send,
    {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
            .await?;
        self.flush(file).await.map_err(Error::Serde)?;
        Ok(())
    }

    /// Delete the data structure from disk.
    ///
    /// Override this method if you wish to perform extra cleanup before deletion.
    async fn delete(path: impl AsRef<Path> + Send + Sync) -> Result<(), Error<Self::Err>> {
        fs::remove_file(path.as_ref()).await?;
        Ok(())
    }
}
