//! A client for fetching FeoBlog data from a server.
//! 
//! The goal is that this can be used from a binary (CLI/server), 
//! or WebAssembly, or TypeScript/JavaScript.


use protobufs::feoblog::ItemList;
pub use protobufs::protobuf;
pub use protobufs::feoblog as protobuf_types;
use protobufs::protobuf::Message;
use protobufs::protobuf::ProtobufError;

mod tests;

#[derive(Debug, Clone)]
pub struct ClientArgs {
    pub base_url: String,
}

pub struct Client {
    base_url: String
}

impl Client {
    pub fn new(args: ClientArgs) -> Self {
        Self {
            base_url: args.base_url.trim_end_matches("/").to_owned(),
        }
    }

    pub async fn get_homepage(&self, args: GetHomepageArgs) -> Result<ItemList> {
        let url = format!("{}/homepage/proto3", self.base_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);

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
        let mut items = ItemList::new();
        items.merge_from_bytes(&body)?;

        Ok(items)
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

#[derive(Debug, Default)]
pub struct GetHomepageArgs {
    pub begin_at: Option<FetchDirection>,
    // TODO: max_count: Optional<u32>,
}

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