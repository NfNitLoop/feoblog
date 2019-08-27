#[cfg(test)]
mod test;

/// Given the private half of a signing key, recalculate the public key:
/// See: https://bitcoin.stackexchange.com/questions/42437/how-to-generate-ed25519-public-key-from-private-key-using-libsodium
pub(crate) fn derive_pk(sk_bytes: &[u8]) -> Vec<u8>
{
    use rust_base58::*;

    use rust_sodium::crypto::sign;

    // crypto_sign_ed25519_sk_to_seed is only in the unsafe FFI crate. :~(
    // use rust_sodium_sys::crypto_sign_ed25519_sk_to_seed;
    use rust_sodium_sys::{crypto_sign_ed25519_sk_to_seed, crypto_sign_seed_keypair};

    let mut seed: Vec<u8> = vec![0; sign::SEEDBYTES];
    let mut pk: Vec<u8> = vec![0; sign::PUBLICKEYBYTES];
    let mut combined_sk: Vec<u8> = vec![0; sign::SECRETKEYBYTES]; // pk+sk combined into one.

    // crypto_sign_ed25519_sk_to_pk just pulls the pk part of a combined sk+pk. (or garbage).
    // To really use the sk half, we have to derive the seed and regen from there:
    // Thanks to: https://forum.nem.io/t/getting-public-key-from-private-key-via-libnacl-libsodium-python/11521/4
    unsafe {
        crypto_sign_ed25519_sk_to_seed(seed.as_mut_ptr(), sk_bytes.as_ptr());
        crypto_sign_seed_keypair(pk.as_mut_ptr(), combined_sk.as_mut_ptr(), seed.as_ptr());
    }

    return pk;


// DOES NOT WORK:
//    let mut output = vec![0; sign::PUBLICKEYBYTES];
//    let result;
//    unsafe {
//        result = crypto_sign_ed25519_sk_to_pk(output.as_mut_ptr(), bytes.as_ptr());
//    }
//    println!("result: {}", result);
//    return output;


}
