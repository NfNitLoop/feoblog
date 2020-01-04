//! Types for data storage/retrieval.

pub(crate) mod sqlite;

use failure::{Error, ResultExt, bail};

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

    fn get_blob(&self, key: &Hash) -> Result<Option<Vec<u8>>, Error>;

    fn get_signature(&self, key: &[u8]) -> Result<Option<Signature>, Error>;

    fn save_blob(&self, data: &[u8]) -> Result<Hash, Error>;

    fn get_hashes(&self) -> Result<Vec<Hash>, Error>;
}

/// The signature is the top-level data structure for the backend.
/// Everything posted must have an associated signature.
/// These are stored in the "signatures" table.
pub struct Signature
{
    /// An ed25519 signature bytes.
    pub signature: Vec<u8>,

    /// An ed25519 public key.
    pub user_id: Vec<u8>,

    /// The multihash of the metadata for this item.
    pub metadata_hash: Hash,
}

// A multihash
pub struct Hash
{
    pub multihash: Vec<u8>
}

/// Mutliash!
impl Hash
{
    /// Construct a multihash of the preferred type for the content.
    pub fn calculate(bytes: &[u8]) -> Self
    {
        use multihash::{encode, Hash as Alg};
        let hash = encode(Alg::SHA2256, bytes).expect(
            "AFAICT this can't actually fail"
        );

        Hash{ multihash: hash }
    }

    pub fn as_bytes(&self) -> &[u8] { self.multihash.as_ref() }

    pub fn to_base58(&self) -> String
    {
        use rust_base58::*;
        self.multihash.to_base58()
    }

    pub fn from_base58(base58: &str) -> Result<Hash, Error>
    {
        use rust_base58::*;
        use multihash::{decode, Hash as Alg};

        let bytes = match base58.from_base58() {
            Ok(value) => value,
            Err(err) => bail!("Base54 error: {}", err)
        };
        let mh = decode(bytes.as_ref())
            .context("Invalid multihash")?
        ;
        if mh.alg != Alg::SHA2256 {
            bail!("Unsupported hash algorithm: {:?}", mh.alg);
        }
        Ok(
            Hash{
                multihash: bytes
            }
        )
    }
}