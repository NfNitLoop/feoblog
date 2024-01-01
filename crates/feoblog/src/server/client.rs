//! Code to support the web-based FeoBlog client, served at /client/.

use rust_embed::RustEmbed;

// served from /client/
#[derive(RustEmbed, Debug)]
#[folder = "web-client/build/"]
pub(crate) struct WebClientBuild;