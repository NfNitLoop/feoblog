//! Types for data storage/retrieval.

pub(crate) mod sqlite;

use core::str::FromStr;
use failure::{Error, ResultExt, bail, format_err};
use bs58;
use sodiumoxide::crypto::sign;


/// Knows how to open Backend "connections".
pub trait Factory: Clone
{
    fn open(&self) -> Result<Box<dyn Backend>, Error>;
}

/// Represents a connection to the backend, and logic we want to perform
/// with it.
pub trait Backend
{
    // TODO: Remove reliance on failure::Error. We should define our own error
    // type here. Should probably impl Error, which requires changes in sqlite.
    // Maybe Box<dyn Error> is sufficient? https://github.com/dtolnay/anyhow/issues/25
    
    /// Set up the initial DB state, maybe running migrations.
    fn setup(&self) -> Result<(), Error>;

    fn save_blob(&self, data: &[u8]) -> Result<Hash, Error>;

    /// Find most recent items for users flagged to be displayed on the
    /// home page, which have timestamps before `before`.
    fn homepage_items(&self, before: Timestamp, max_count: u32) -> Result<Vec<ItemRow>, Error>;

    /// Find the most recent items for a particular user
    fn user_items(&self, user: &UserID, before: Timestamp) -> Result<Vec<ItemRow>, Error>;

    /// Find one particular UserItem
    fn user_item(&self, user: &UserID, signature: &Signature) -> Result<Option<ItemRow>, Error>;

    /// Effieicntly check whether a user item exists:
    fn user_item_exists(&self, user: &UserID, signature: &Signature) -> Result<bool, Error>;

    /// Save an uploaded item to the data store. 
    fn save_user_item(&self, item: &ItemRow) -> Result<(), Error>;

    /// Get a "server user" -- a user granted direct access to post to the
    /// server.
    fn server_user(&self, user: &UserID) -> Result<Option<ServerUser>, Error>;

    /// Reads an entire blob into memory. TODO: Make a streaming version.
    fn get_blob(&self, key: &Hash) -> Result<Option<Vec<u8>>, Error>;
}

/// A UserID is a nacl public key. (32 bytes)
#[derive(Clone)]
pub struct UserID {
    pub_key: sign::PublicKey,
}

// Expect a 32-byte nacl public key:
const USER_ID_BYTES: usize = 32;

impl UserID {
    pub fn to_base58(&self) -> String {
        bs58::encode(self.bytes()).into_string()
    }

    pub fn from_base58(value: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(value).into_vec()?;
        Self::from_vec(bytes)
    }

    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() != USER_ID_BYTES {
            bail!("Expected {} bytes but found {}", USER_ID_BYTES, bytes.len());
        }

        let pub_key = sign::PublicKey::from_slice(&bytes).ok_or_else(
            || format_err!("Error creating nacl::PuublicKey")
        )?;

        Ok( UserID{ pub_key } )
    }

    pub fn bytes(&self) -> &[u8] {
        self.pub_key.as_ref()
    }
}

/// Allows easy destructuring from URLs.
impl FromStr for UserID {
    type Err = failure::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { 
        UserID::from_base58(value)
    }
}

/// Bytes representing a detached NaCl signature. (64 bytes)
#[derive(Clone)]
pub struct Signature {
    signature: sign::Signature,
}

const SIGNATURE_BYTES: usize = 64;

impl Signature {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() != SIGNATURE_BYTES {
            bail!("Signature expected {} bytes but found {}", SIGNATURE_BYTES, bytes.len());
        }

        let signature = sign::Signature::from_slice(&bytes).ok_or_else(
            || format_err!("Failure creating nacl::Signature")
        )?;
        
        Ok( Signature{ signature } )
    }

    pub fn from_base58(value: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(value).into_vec()?;
        Self::from_vec(bytes)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.bytes()).into_string()
    }

    pub fn bytes(&self) -> &[u8] {
        self.signature.as_ref()
    }

    /// True if this signature is valid for the given user on the given bytes.
    pub fn is_valid(&self, user: &UserID, bytes: &[u8]) -> bool {
        let pubkey = sign::PublicKey::from_slice(user.bytes()).expect("pubkey");
        sign::verify_detached(&self.signature, bytes, &pubkey)
    }

}

/// Allows easy destructuring from URLs.
impl FromStr for Signature {
    type Err = failure::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { 
        Signature::from_base58(value)
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

// pub struct ItemRowIter<'bend, Error> 
// {
//     next_fn: &'bend dyn FnMut() -> Option<Result<ItemRow, Error>>
// }

// impl <'bend, Error> ItemRowIter<'bend, Error>
// {
//     fn new(f: NextFn) -> Self {
//         ItemRowIter{ next_fn: f }
//     }
// }

// impl <F, Error> Iterator for ItemRowIter<F, Error>
// where F: FnMut() -> Option<Result<ItemRow, Error>>
// {
//     type Item = Result<ItemRow, Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         (self.next_fn)()
//     }
// }

/// Info about users explicitly allowed on this server.
/// i.e.: A row in the server_user table.
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
    pub fn now() -> Self {
        use time::OffsetDateTime;
        let delta = OffsetDateTime::now_utc() - OffsetDateTime::unix_epoch();
        Timestamp {
            unix_utc_ms: delta.whole_milliseconds() as i64,
        }
    }
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

