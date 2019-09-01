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

    let (dsk, dpk) = derive_pk(vecsk.as_slice());
    assert_eq!(pk, dpk);

}

#[test]
fn test_high_level() {
    use crate::crypto::*;

    let pair = SigKeyPair::new();
    let derived = SigKeyPair::from(pair.secret());
    assert_eq!(pair, derived);

    let data: Vec<u8> = "Hello, world!".into();

    let sig = pair.sign(data.as_slice());
    let valid = pair.public().validate(data.as_slice(), &sig);

    assert_eq!(true, valid);
}