use std::{str::FromStr, fmt::Display};

use dryoc::sign;
use dryoc::classic::crypto_sign::crypto_sign_verify_detached as verify_detached;
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// TryFrom errors for [`UserID`] and [`Signature`]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("Network error")]
    // Reqwest(#[from] reqwest::Error),

    // #[error("Error decoding protobuf")]
    // Protobuf(#[from] ProtobufError),

    #[error("Error decoding base58 text")]
    Base58Decode(#[from] bs58::decode::Error),

    #[error("Wrong number of bytes")]
    WrongNumBytes(#[from] dryoc::Error),
}

/// A UserID is a nacl public key. (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct UserID {
    pub_key: sign::PublicKey,
}

impl std::hash::Hash for UserID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pub_key.hash(state);
    }
}

impl UserID {
    pub fn to_base58(&self) -> String {
        bs58::encode(self.bytes()).into_string()
    }

    pub fn from_base58(value: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(value).into_vec()?;
        Self::from_vec(bytes)
    }

    // TODO: Deprecate.
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        Self::try_from(bytes.as_slice())
    }

    pub fn bytes(&self) -> &[u8] {
        self.pub_key.as_ref()
    }
}

impl TryFrom<&[u8]> for UserID {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let pub_key = sign::PublicKey::try_from(bytes)?;
        Ok( UserID{ pub_key } )
    }
}

/// Parse from the base58 representation.
impl FromStr for UserID {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { 
        UserID::from_base58(value)
    }
}

impl Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

/// Bytes representing a detached NaCl signature. (64 bytes)
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Signature {
    signature: sign::Signature,
}

impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signature.hash(state);
    }
}

impl Signature {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, Error> {
        let signature = sign::Signature::try_from(bytes.as_slice())?;        
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
        verify_detached(self.signature.as_ref(), bytes, &user.pub_key.as_ref()).is_ok()
    }

}

/// Parse from the base58 representation.
impl FromStr for Signature {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> { 
        Signature::from_base58(value)
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let signature = sign::Signature::try_from(bytes)?;
        Ok( Signature{ signature } )
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}