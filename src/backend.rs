//! Types for data storage/retrieval.

pub(crate) mod sqlite;

use failure::Error;

/// Knows how to open Backend "connections".
pub trait Factory: Clone
{
    fn open(&self) -> Result<Box<dyn Backend>, Error>;
}

/// Represents a connection to the backend, and logic we want to perform
/// with it.
pub trait Backend
{
    // Set up the initial DB state, maybe running migrations.
    fn setup(&self) -> Result<(), Error>;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
}
