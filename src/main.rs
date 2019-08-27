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
        Crypto{..} => test_crypto(),
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

    #[structopt(
        name="crypto",
        raw(setting = "structopt::clap::AppSettings::Hidden"),
    )]
    /// Test some crypto primitives.
    Crypto { },
}

fn test_crypto() -> Result<(), Error>
{
    use rust_sodium::crypto::sign::{gen_keypair, sign_detached};
    use rust_base58::*;

    let (ourpk, oursk) = gen_keypair();
    let vecpk: Vec<u8> = ourpk[..].into();
    let vecsk: Vec<u8> = oursk[..32].into();
//    println!("pk: {:#?}", &vecpk);
//    println!("sk: {:#?}", &vecsk);

    println!("pk: {}, bytes: {}", ourpk[..].to_base58(), vecpk.len());
    println!("sk: {}, bytes: {}", oursk[..].to_base58(), vecsk.len());
    // println!("sk.pk: {}", oursk[32..64].to_base58());

    use crate::crypto::derive_pk;
    println!("derived pk: {}", derive_pk(&vecsk[0..32]).to_base58());

    let text: Vec<u8> = "Hello, world!".into();
    let signature = sign_detached(&text[..], &oursk);
    println!("sig: {}", signature[..].to_base58());

    let mut combined = Vec::new();
    combined.extend_from_slice(&ourpk[..]);
    combined.extend_from_slice(&signature[..]);
    println!("combined: {}", combined.to_base58());


    use rust_sodium::crypto::sign::*;
    println!("PUBLICKEYBYTES: {}", PUBLICKEYBYTES);
    println!("SECRETKEYBYTES: {}", SECRETKEYBYTES);
    println!("SIGNATUREBYTES: {}", SIGNATUREBYTES);

    return Ok(());
}
