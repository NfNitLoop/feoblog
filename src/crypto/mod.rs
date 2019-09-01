use rust_sodium::crypto::sign;
use std::borrow::Cow;


#[cfg(test)]
mod test;



/// A keypair for signing.
#[derive(Debug,Eq,PartialEq)]
pub(crate) struct SigKeyPair {
    // rust_sodium's secret key actually contains both.
    combined: sign::SecretKey,

    // While the above contains the PublicKey, the rust_sodium API requires an instance of this
    // to do validations.
    public: SigPublicKey,
}


impl SigKeyPair {

    fn new() -> Self {
        let (pk, sk) = sign::gen_keypair();
        Self {
            combined: sk,
            public: SigPublicKey{ nacl_key: pk },
        }
    }

    pub fn public(&self) -> &SigPublicKey {
        &self.public
    }

    pub fn secret(&self) -> SigSecretKey {
        SigSecretKey {
            bytes: Cow::from(&self.combined[..SEC_KEY_BYTES]),
        }
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        Signature {
            nacl_sig: sign::sign_detached(data, &self.combined)
        }
    }
}

impl<'a> From<SigSecretKey<'a>> for SigKeyPair {
    fn from(ssk: SigSecretKey) -> Self {
        let SigSecretKey{bytes: sk_bytes} = ssk;
        let (sk, pk) = derive_pk(&sk_bytes[..]);
        Self {
            combined: sk,
            public: SigPublicKey{nacl_key: pk},
        }
    }
}

/// Unlike the secret key in rust_sodium, this one only contains the secret half, since
/// we can recompute the full key.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SigSecretKey<'a> {
    bytes: Cow<'a, [u8]>,
}

/// rust_sodium secret keys contain the public one, but we trim it:
const SEC_KEY_BYTES: usize = sign::SECRETKEYBYTES - sign::PUBLICKEYBYTES;
const PUB_KEY_BYTES: usize = sign::PUBLICKEYBYTES;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SigPublicKey {
    nacl_key: sign::PublicKey
}

impl SigPublicKey {
    pub fn validate(&self, data: &[u8], sig: &Signature) -> bool {
        sign::verify_detached(&sig.nacl_sig, data, &self.nacl_key)
    }
}

pub struct Signature {
    nacl_sig: sign::Signature,
}

impl Signature {
    // TODO:  fn from_bytes(bytes: &[u8])
}

/// Given the private half of a signing key, recalculate the public key:
/// See: https://bitcoin.stackexchange.com/questions/42437/how-to-generate-ed25519-public-key-from-private-key-using-libsodium
pub(crate) fn derive_pk(sk_bytes: &[u8]) -> (sign::SecretKey, sign::PublicKey)
{
    // crypto_sign_ed25519_sk_to_seed is only in the unsafe FFI crate. :~(
    use rust_sodium_sys::{crypto_sign_ed25519_sk_to_seed, crypto_sign_seed_keypair};

    let mut seed: Vec<u8> = vec![0; sign::SEEDBYTES];
    let mut pk = [0 as u8; sign::PUBLICKEYBYTES];
    let mut combined_sk = [0 as u8; sign::SECRETKEYBYTES]; // pk+sk combined into one.

    // crypto_sign_ed25519_sk_to_pk just pulls the pk part of a combined sk+pk. (or garbage).
    // To really use the sk half, we have to derive the seed and regen from there:
    // Thanks to: https://forum.nem.io/t/getting-public-key-from-private-key-via-libnacl-libsodium-python/11521/4
    // TODO: Update to safe versions of these calls if they ever get added to rust_sodium.
    unsafe {
        crypto_sign_ed25519_sk_to_seed(seed.as_mut_ptr(), sk_bytes.as_ptr());
        crypto_sign_seed_keypair(pk.as_mut_ptr(), combined_sk.as_mut_ptr(), seed.as_ptr());
    }

    (sign::SecretKey(combined_sk), sign::PublicKey(pk))
}
