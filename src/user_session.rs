use crate::crypto::{SigKeyPair, SigPublicKey};
use failure; 
use rust_base58::base58::FromBase58;

pub(crate) trait UserSession {
    fn logged_in(&self) -> bool;

    fn log_out(&self);

    fn log_in(&self, secret_key: &[u8]) -> Result<(), failure::Error>;

    fn pub_key(&self) -> Option<SigPublicKey>;
}

// Names we use to store the values in the Session:
const SECRET_KEY: &'static str = "secret_key";

