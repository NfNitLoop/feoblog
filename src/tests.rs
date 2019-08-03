
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
    serde_cbor::to_writer(&mut out_bytes, &input);

    let out_foo: Foo = serde_cbor::from_reader(&out_bytes[..]).unwrap();
    let out_bar: Bar = serde_cbor::from_reader(&out_bytes[..]).unwrap();

    assert_eq!(input.message, out_foo.message);
    assert_eq!(input.number, out_foo.number);

    assert_eq!(input.message, out_bar.message);
    assert_eq!(input.number, out_bar.number);
}
