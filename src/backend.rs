//! Types for data storage/retrieval.

pub(crate) mod sqlite;

use failure::{Error, ResultExt, bail};
use bs58;
use rust_sodium::crypto::sign;


/// Knows how to open Backend "connections".
pub trait Factory: Clone
{
    fn open(&self) -> Result<Box<dyn Backend>, Error>;
}

/// Represents a connection to the backend, and logic we want to perform
/// with it.
pub trait Backend
{
    /// Set up the initial DB state, maybe running migrations.
    fn setup(&self) -> Result<(), Error>;

    fn save_blob(&self, data: &[u8]) -> Result<Hash, Error>;

    /// Find most recent items for users flagged to be displayed on the
    /// home page, which have timestamps before `before`.
    fn homepage_items(&self, before: Timestamp) -> Result<Vec<ItemRow>, Error>;

    /// Find the most recent items for a particular user
    fn user_items(&self, user: &UserID, before: Timestamp) -> Result<Vec<ItemRow>, Error>;

    /// Save an uploaded item to the data store. 
    fn save_user_item(&self, item: ItemRow) -> Result<(), Error>;

    /// Get a "server user" -- a user granted direct access to post to the
    /// server.
    fn server_user(&self, user: &UserID) -> Result<Option<ServerUser>, Error>;

    /// Reads an entire blob into memory. TODO: Make a streaming version.
    fn get_blob(&self, key: &Hash) -> Result<Option<Vec<u8>>, Error>;
}

/// A UserID is a nacl public key. (32 bytes)
#[derive(Clone)]
pub struct UserID {
    bytes: Vec<u8>
}

// Expect a 32-byte nacl public key:
const USER_ID_BYTES: usize = 32;

impl UserID {
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.bytes).into_string()
    }

    pub fn from_base58(value: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(value).into_vec()?;
        Self::from_vec(bytes)
    }

    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() != USER_ID_BYTES {
            bail!("Expected {} bytes but found {}", USER_ID_BYTES, bytes.len());
        }

        Ok(
            UserID{ bytes: bytes }
        )
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

/// Bytes representing a detached NaCl signature. (64 bytes)
#[derive(Clone)]
pub struct Signature {
    bytes: Vec<u8>
}

const SIGNATURE_BYTES: usize = 64;

impl Signature {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() != SIGNATURE_BYTES {
            bail!("Expected {} bytes but found {}", SIGNATURE_BYTES, bytes.len());
        }

        Ok(
            Signature{ bytes: bytes }
        )
    }

    pub fn from_base58(value: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(value).into_vec()?;
        Self::from_vec(bytes)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.bytes).into_string()
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// True if this signature is valid for the given user on the given bytes.
    pub fn is_valid(&self, user: &UserID, bytes: &[u8]) -> bool {
        let signature = sign::Signature::from_slice(self.bytes()).expect("sig");
        let pubkey = sign::PublicKey::from_slice(user.bytes()).expect("pubkey");
        sign::verify_detached(&signature, bytes, &pubkey)
    }

}

/// Data that should be stored along with an Item
/// 
/// The signature should be validated on the front-end before being
/// sent to the back-end. (This avoids each back-end having to re-implement
/// validation logic). Likewise, the front-end may want to validate data returned
/// by the backend to ensure it hasn't been modified or bit-rot.
pub struct ItemRow {
    pub user: UserID,
    pub signature: Signature,

    // The (signed) timestamp from within item_bytes.
    pub timestamp: Timestamp,
    
    /// The time that this item was received by the server.
    pub received: Timestamp,

    /// Bytes which can be deserialized into an Item.
    pub item_bytes: Vec<u8>
}

pub struct ServerUser {
    user: UserID,
    notes: String,
    on_homepage: bool,
}


#[derive(Copy, Clone)]
pub struct Timestamp {
    /// UNIX time, at UTC, in milliseconds:
    pub unix_utc_ms: i64
}

impl Timestamp {
    // TODO: now()
}

// A multihash
pub struct Hash
{
    pub multihash: Vec<u8>
}

// TODO: Get rid of this.
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

