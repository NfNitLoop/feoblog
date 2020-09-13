#![deny(unknown_lints)]
#![deny(unused_must_use)]

#[cfg(test)]
mod tests;

use std::io;

use failure::{Error, bail, ResultExt};
use structopt::StructOpt;

mod backend;
mod protos;
mod responder_util;
mod server;


fn main() -> Result<(), Error> {
    let command = Command::from_args();
    use Command::*;

    let options = match command {
        Open{mut shared_options} => {
            shared_options.allow_login = true;
            shared_options
        },
        Serve{shared_options} => shared_options,
        other => {
            bail!("Unimplemented: {:?}", other);
        }
    };

    server::serve(options)
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
    Open {
        #[structopt(flatten)]
        shared_options: SharedOptions,
     },

    #[structopt(name="serve")]
    /// Serve local content as a web site.
    /// The write UI is disabled. Content must be signed and pushed from
    /// other instances.
    // TODO: Delete. There's no longer a distinction between serve and open,
    // since we're removing login sessions from the server.
    Serve { 
        #[structopt(flatten)]
        shared_options: SharedOptions,
    },
}

#[derive(StructOpt, Debug, Clone)]
pub(crate) struct SharedOptions
{
    #[structopt(default_value = "feoblog.sqlite3")]
    pub sqlite_file: String,

    #[structopt(long)]
    /// This flag is toggled on by the "open" subcommand.
    /// Generally, you do not want to let people log in to your server,
    /// as it requires them to send you their secret key -- behavior which we
    /// don't want to encourage.
    pub allow_login: bool 
}