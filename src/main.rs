#![deny(unknown_lints)]
#![deny(unused_must_use)]

#[cfg(test)]
mod tests;

use std::io;

use failure::{Error, bail, ResultExt};
use structopt::StructOpt;

mod backend;
mod crypto;
mod responder_util;
mod server;


fn main() -> Result<(), Error> {
    let command = Command::from_args();
    use Command::*;

    match command {
        Open{..} => server::cmd_open(),
        other => {
            bail!("Unimplemented: {:?}", other);
        }
    }
}


#[derive(StructOpt, Debug)]
#[structopt(
    name="feoblog",
    about="A distributed P2P (micro|macro)blog system.",
    author="",
)]
enum Command
{
    #[structopt(name="open")]
    /// Open a browser window to locally view/add content.
    Open { },

    #[structopt(name="serve")]
    /// Serve local content as a web site.
    /// The write UI is disabled. Content must be signed and pushed from
    /// other instances.
    Serve { },
}


