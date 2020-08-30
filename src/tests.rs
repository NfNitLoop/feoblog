
// Test that the serialization/deserialization is not based on the order of
// structs' fields, but their names.
// This may seem ridiculous, but I almost found out the hard way that that's 
// not the case with the Rust MessagePack implementation.
// I ended up preferring CBOR anyway. :) 
#[test]
fn unordered_serialization() {
    use serde_derive::{Deserialize,Serialize};

    #[derive(Serialize,Deserialize)]
    struct Foo {
        message: String,
        number: u32,
    }
    #[derive(Deserialize)]
    struct Bar {
        number: u32,
        message: String,
    }

    let input = Foo{
        message: "Hello, World!".to_string(),
        number: 42
    };
    let mut out_bytes = Vec::new();
    serde_cbor::to_writer(&mut out_bytes, &input).expect("cbor.to_writer()");

    let out_foo: Foo = serde_cbor::from_reader(&out_bytes[..]).unwrap();
    let out_bar: Bar = serde_cbor::from_reader(&out_bytes[..]).unwrap();

    assert_eq!(input.message, out_foo.message);
    assert_eq!(input.number, out_foo.number);

    assert_eq!(input.message, out_bar.message);
    assert_eq!(input.number, out_bar.number);
}

// TODO: These are equivalent and bs58 seems better. migrate.
// Test that our base58 encoder can encode/decode arbitrary bytes.
#[test]
fn base58_bytes() {
    use rust_base58::*;

    // Public keys and the private half of private keys are 32 bytes:
    let original_bytes = vec![
        0x00, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
    ];
    
    let encoded = original_bytes.to_base58();

    assert_eq!(encoded.as_str(), "11t6ZcNTcApL3s8ScmkCwNcFxJ47FfATY7cdg632k4X");
    let decoded = encoded.from_base58().unwrap();

    assert_eq!(original_bytes, decoded);

    // // Typos get decoded too?  Nope.
    // let decoded = "llLQX".from_base58().unwrap();
    // assert_eq!(original_bytes, decoded);
}

// The new one is better?
#[test]
fn bs58_bytes() {
    use bs58;

    // Public keys and the private half of private keys are 32 bytes:
    let original_bytes = vec![
        0x00, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0xFF, 0x00,
    ];
    
    let encoded = bs58::encode(&original_bytes)
        .with_alphabet(bs58::alphabet::BITCOIN)
        .into_string()
    ;

    assert_eq!(encoded.as_str(), "11t6ZcNTcApL3s8ScmkCwNcFxJ47FfATY7cdg632k4X");


    let decoded = bs58::decode(encoded)
        .into_vec()
        .unwrap()
    ;

    assert_eq!(original_bytes, decoded);


    let encoded = bs58::encode(&original_bytes)
        .with_check()
        .into_string()
    ;

    assert_eq!(encoded.as_str(), "116mMWNtNGD53Go3cNrwfPGW8dDEgLCGPdB7HAND77SUGmaWE");

    // // Should be able to fix up incorrect letters: But does not.
    // let decoded = bs58::decode("1l6mMWNtNGD53Go3cNrwfPGW8dDEgLCGPdB7HAND77SUGmaWE")
    //     .with_check(None)
    //     .into_vec()
    //     .unwrap();
    // assert_eq!(original_bytes, decoded);

}
