//! Types for data storage/retrieval.

pub(crate) mod sqlite;

use crate::protos::Item;
use core::str::FromStr;
use std::marker::PhantomData;
use failure::{Error, ResultExt, bail, format_err};
use bs58;
use serde::{Deserialize, de::{self, Visitor}};
use sodiumoxide::crypto::sign;

/// This trait knows how to build a Factory, which in turn can open Backend connections.
///
/// It also provides functionality for checking/upgrading the backing database.
pub trait FactoryBuilder {
    /// Create a new factory which is capable of opening Backends.
    /// Must first check that the database exists, and is the correct version.
    fn factory(&self) -> Result<Box<dyn Factory>, Error>;

    fn db_exists(&self) -> Result<bool, Error>;

    fn db_create(&self) -> Result<(), Error>;

    fn db_needs_upgrade(&self) -> Result<bool, Error>;

    /// Upgrade the database to the currently supported version.
    fn db_upgrade(&self) -> Result<(), Error>;
}
/// Knows how to open Backend "connections".
pub trait Factory: Send
{
    /// Create a clone of this Factory.
    /// Like Clone, but can operate on dyn pointers.
    fn dyn_clone(&self) -> Box<dyn Factory>;

    /// Open a single Backend connection.
    /// It is recommended that Factory implementions use their own connection pooling.
    fn open(&self) -> Result<Box<dyn Backend>, Error>;
}

/// Dumb hack to make dyn Factory impl Cloneable
/// Clone is required for passing Factory instances to multiple web server threads.
pub struct FactoryBox {
    pub factory: Box<dyn Factory>
}

impl Clone for FactoryBox {
    fn clone(&self) -> Self {
        Self {
            factory: self.factory.dyn_clone()
        }
    }
}

/// Represents a connection to the backend, and logic we want to perform
/// with it.
pub trait Backend
{
    // TODO: Remove reliance on failure::Error. We should define our own error
    // type here. Should probably impl Error, which requires changes in sqlite.
    // Maybe Box<dyn Error> is sufficient? https://github.com/dtolnay/anyhow/issues/25
    
    /// Find most recent items for users flagged to be displayed on the
    /// home page, which have timestamps before `before`.
    /// Items are returned through callback, and will continue to be fetched while callback continues
    /// to return Ok(true).
    fn homepage_items<'a>(&self, before: Timestamp, callback: &'a mut dyn FnMut(ItemDisplayRow) -> Result<bool,Error>) -> Result<(), Error>;

    /// Find the most recent items for a particular user
    fn user_items<'a>(
        &self,
        user: &UserID,
        before: Timestamp,
        callback: RowCallback<'a, ItemRow>,
    ) -> Result<(), Error>;

    /// Most recent replies to an Item
    fn reply_items<'a>(
        &self,
        user: &UserID,
        signature: &Signature,
        before: Timestamp,
        callback: RowCallback<'a, ItemRow>,
    ) -> Result<(), Error>;

    /// Find the most recent items from users followed by the given user ID. Includes the users's own items too.
    fn user_feed_items<'a>(
        &self,
        user_id: &UserID,
        before: Timestamp,
        callback: RowCallback<'a, ItemDisplayRow>,
    ) -> Result<(), Error>;

    /// Find one particular UserItem
    fn user_item(&self, user: &UserID, signature: &Signature) -> Result<Option<ItemRow>, Error>;

    /// Effieicntly check whether a user item exists:
    fn user_item_exists(&self, user: &UserID, signature: &Signature) -> Result<bool, Error>;

    /// Save an uploaded item to the data store.
    fn save_user_item(&mut self, item_row: &ItemRow, item: &Item) -> Result<(), Error>;

    /// Get a "server user" -- a user granted direct access to post to the
    /// server.
    fn server_user(&self, user: &UserID) -> Result<Option<ServerUser>, Error>;

    /// List users granted direct access to post to the server.
    fn server_users<'a>(&self, cb: RowCallback<'a, ServerUser>) -> Result<(), Error>;

    /// Add a new "server user" who is explicitly allowed to post to this server.
    fn add_server_user(&self, server_user: &ServerUser) -> Result<(), Error>;

    /// Get the Item(Row) that represents the user's most recently saved profile, if it exists.
    fn user_profile(&self, user_id: &UserID) -> Result<Option<ItemRow>, Error>;

    /// Is this user ID known to this server?
    ///
    /// This is true if any of these are true:
    /// * The user is a "server user" (given direct permission to post to this server)
    /// * The user is followed by a "server user". (We want their content so we can create a feed.)
    fn user_known(&self, user_id: &UserID) -> Result<bool, Error>;

    /// Check whether a user has remaiing quota/permissions to upload a particular item.
    fn quota_check_item(&self, user_id: &UserID, bytes: &[u8], item: &Item) -> Result<Option<QuotaDenyReason>, Error>;
}

/// A callback function used for callback iteration through large database resultsets.
/// Each row T will be sent to the callback. The callback should return Ok(true) to continue iteration.
pub type RowCallback<'a, T> = &'a mut dyn FnMut(T) -> Result<bool, Error>; 

/// A UserID is a nacl public key. (32 bytes)
#[derive(Debug, Clone)]
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

/// Allows easy destructuring from URLs. (in Warp)
impl FromStr for Signature {
    type Err = failure::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { 
        Signature::from_base58(value)
    }
}

impl <'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> 
    {
        deserializer.deserialize_str(FromStrVisitor::<Self>::new())
    }
}

impl <'de> Deserialize<'de> for UserID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> 
    {
        deserializer.deserialize_str(FromStrVisitor::<Self>::new())
    }
}

struct FromStrVisitor<T: FromStr> {
    _t: PhantomData<T>
}

impl <T: FromStr> FromStrVisitor<T> {
    fn new() -> Self {
        FromStrVisitor { _t: PhantomData }
    }
}

impl <'de, T: FromStr<Err=Error>> Visitor<'de> for FromStrVisitor<T> 
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a &str that can be converted to a {}",
            std::any::type_name::<T>()
        )
    }

    fn visit_str<E>(self, v: &str)
    -> Result<Self::Value, E>
    where E: de::Error
    {
        T::from_str(v).map_err(|e| de::Error::custom(format!("{}", e.compat())))
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
    pub item_bytes: Vec<u8>,
}

/// An [`ItemRow`] that has extra information (fetched via joins)
pub struct ItemDisplayRow {
    pub item: ItemRow,

    /// The display name for the author of the item, if available.
    pub display_name: Option<String>
}

/// Profile information from the `profile` table. `profile` table.
/// Expected to be fetched via join/query on userID, so that's excluded.
pub struct Profile {
    /// The signature for the Item that contains the latest profile.
    pub signature: Signature,

    /// May be empty if the user omitted a display name.
    pub display_name: String,
}


/// Info about users explicitly allowed on this server.
/// i.e.: A row in the server_user table.
#[derive(Debug, Clone)]
pub struct ServerUser {
    pub user: UserID,
    pub notes: String,
    pub on_homepage: bool,
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

    pub fn format_with_offset(self, minutes: i16) -> String {
        use time::{Duration, UtcOffset, OffsetDateTime};
        use std::ops::Add;

        let ms = Duration::milliseconds(self.unix_utc_ms);
        let datetime = OffsetDateTime::unix_epoch().add(ms);
        let offset = UtcOffset::minutes(minutes);
        let datetime = datetime.to_offset(offset);

        datetime.format("%Y-%m-%d %H:%M:%S %z")
    }
}
/// A reason why a user can't post an Item or file attachment.
pub enum QuotaDenyReason {
    /// The user already has enough items newer than this one such that posting this one would exceed the quota.
    /// 
    // TODO: Use this.
    #[allow(dead_code)]
    NewerItemsExceedQuota {
        /// The maximum bytes of Items this user can store on the server.
        max_bytes: u64,
    },

    /// This user is not known to the server, so not allowed to post.
    UnknownUser,

    /// We already have a profile that proves that this userID has been revoked.
    ProfileRevoked,
}

impl std::fmt::Display for QuotaDenyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewerItemsExceedQuota { max_bytes } => 
                write!(f, "Newer items exceed {} byte quota.", max_bytes),
            Self::UnknownUser => 
                write!(f, "This user is not known to the server."),
            Self::ProfileRevoked => 
                write!(f, "This user ID has been revoked."),
        }
    }
}