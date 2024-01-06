//! A client for fetching FeoBlog data from a server.
//! 
//! The goal is that this can be used from a binary (CLI/server), 
//! or WebAssembly, or TypeScript/JavaScript.

mod tests;
mod types;

use protobufs::feoblog::Item;
pub use protobufs::protobuf;
pub use protobufs::feoblog as protobuf_types;
use serde::Deserialize;
use serde::Serialize;
pub use types::{Signature, UserID};

use protobufs::feoblog::ItemList;
use protobufs::protobuf::Message;
use protobufs::protobuf::ProtobufError;



#[derive(Debug, Clone)]
pub struct ClientArgs {
    pub base_url: String,
}

pub struct Client {
    base_url: String,
    rq: reqwest::Client,
}

impl Client {
    pub fn new(args: ClientArgs) -> Self {
        Self {
            base_url: args.base_url.trim_end_matches("/").to_owned(),
            rq: reqwest::Client::new(),
        }
    }

    pub async fn get_homepage(&self, args: GetHomepageArgs) -> Result<ItemList> {
        let url = format!("{}/homepage/proto3", self.base_url);
        let mut request = self.rq.get(&url);

        if let Some(beginning) = args.begin_at {
            match beginning {
                FetchDirection::Before(FetchPosition { timestamp: Timestamp { ms_utc }}) => {
                    request = request.query(&[("before", ms_utc)]);
                },
                FetchDirection::After(FetchPosition{ timestamp: Timestamp { ms_utc }}) => {
                    request = request.query(&[("after", ms_utc)]);
                }
            }
        }

        let response = request.send().await?;
        let body = response.bytes().await?;
        let items = ItemList::parse_from_bytes(&body)?;

        Ok(items)
    }

    pub async fn get_item(&self, user_id: &UserID, sig: &Signature) -> Result<ItemResponse> {
        let url = format!("{}/u/{user_id}/i/{sig}/proto3", self.base_url);
        let req = self.rq.get(&url);
        let response = req.send().await?;
        let bytes = response.bytes().await?;
        let item = Item::parse_from_bytes(&bytes)?;

        Ok(ItemResponse {
            item, 
            bytes: bytes.into(),
            user_id: user_id.clone(),
            signature: sig.clone(),
        })
    }
}

/// Contains not only the `Item` that we fetched from the server,
/// but also the original Protobuf bytes, which we can use to validate the item.
/// 
/// You may choose to skip validation, for example if you trust your own data source
/// or need to delay validation until later. 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemResponse {
    pub item: Item,
    pub bytes: Vec<u8>,
    pub user_id: UserID,
    pub signature: Signature,
}

impl ItemResponse {
    /// Check whether the signature matches the bytes we fetched.
    pub fn is_valid(&self) -> bool {
        self.signature.is_valid(&self.user_id, &self.bytes)
    }
}



/// Represents the kinds of errors that [`Client`] can return.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Network error")]
    Reqwest(#[from] reqwest::Error),

    #[error("Error decoding protobuf")]
    Protobuf(#[from] ProtobufError),
}

pub type Result<T> = std::result::Result<T, Error>;


/// args for [`Client::get_homepage()`]
#[derive(Debug, Default)]
pub struct GetHomepageArgs {
    pub begin_at: Option<FetchDirection>,
    // TODO: max_count: Optional<u32>,
}

/// Which direction we should fetch items from the server.
/// 
/// For an example, see: [`GetHomepageArgs`]
#[derive(Debug)]
pub enum FetchDirection {
    /// Begin at this point in time and go backwards.
    Before(FetchPosition),

    /// Begin at this point and go forward:
    After(FetchPosition),
}

#[derive(Debug)]
pub struct FetchPosition {
    timestamp: Timestamp,
    // TODO: optional Signature.
}

/// A timestmap in ms since the epoch at utc.
#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    pub ms_utc: i64,
}