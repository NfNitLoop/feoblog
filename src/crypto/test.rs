use crate::crypto::derive_pk;

#[test]
fn test_derive_pk() {
    use rust_sodium::crypto::sign::{gen_keypair, sign_detached};
    use rust_base58::*;

    let (pk, sk) = gen_keypair();
    let vecpk: Vec<u8> = pk[..].into();

    // Technically you only need the first 32 bytes of the sk, you can derive the rest.
    // Copy it to a new vec to make sure the library isn't just reading the pk from the sk+pk part.
    // (which is easy to accidentally do because C uses pointers without lengths. Argh!)
    let vecsk: Vec<u8> = sk[..32].into();

    let derived_key = derive_pk(vecsk.as_slice());
    assert_eq!(vecpk, derived_key);

}