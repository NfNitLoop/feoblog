#[cfg(test)]
mod tests;

use std::io;

use failure::{Error, bail, ResultExt};
use structopt::StructOpt;

mod backend;
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
    use rust_sodium::crypto::box_;
    use rust_base58::*;

    let (ourpk, oursk) = box_::gen_keypair();
    let vecpk: Vec<u8> = ourpk[..].into();
    let vecsk: Vec<u8> = oursk[..].into();
    println!("pk: {}, bytes: {}", ourpk[..].to_base58(), vecpk.len());
    println!("sk: {}, bytes: {}", oursk[..].to_base58(), vecsk.len());

    use rust_sodium::crypto::scalarmult::curve25519::Scalar;
    use rust_sodium::crypto::scalarmult::curve25519::scalarmult_base;

    let s = Scalar::from_slice(&oursk[..]).expect("scalar");
    let group_element = scalarmult_base(&s);
    println!("derived pk: {}", group_element[..].to_base58());


    use rust_sodium::crypto::sign::*;
    println!("PUBLICKEYBYTES: {}", PUBLICKEYBYTES);
    println!("SECRETKEYBYTES: {}", SECRETKEYBYTES);
    println!("SIGNATUREBYTES: {}", SIGNATUREBYTES);

    return Ok(());
}
