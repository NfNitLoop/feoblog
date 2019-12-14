use in_memory_session::{Session, SessionReader, SessionWriter};
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

impl UserSession for Session {
    fn logged_in(&self) -> bool {
        let read = self.read();
        let pkey: Option<String> = read.get(SECRET_KEY);
        pkey.is_some()
    }

    fn log_out(&self) {
        let mut writer = self.write();
        writer.clear();
    }

    fn log_in(&self, secret_key: &[u8]) -> Result<(), failure::Error> {
        let keypair = SigKeyPair::from_secret(secret_key)?;

        let mut write = self.write();
        write.set(SECRET_KEY, keypair.secret());

        Ok(())
    }

    fn pub_key(&self) -> Option<SigPublicKey> {
        if !self.logged_in() {
            return None;
        }

        let secret: String = match self.read().get(SECRET_KEY) {
            None => return None,
            Some(string) => string
        };

        let secret: Vec<u8> = match secret.from_base58() {
            Err(err) => return None,
            Ok(bytes) => bytes,
        };

        let keypair = match SigKeyPair::from_secret(secret.as_ref()) {
            Err(err) => return None,
            Ok(keypair) => keypair,
        };
        
        Some(keypair.public().clone())

    }
}