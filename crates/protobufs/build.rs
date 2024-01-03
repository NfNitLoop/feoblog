const PROTO_FILE: &str = "input/feoblog.proto";

const INPUTS: [&str; 2] = [
    "build.rs",
    PROTO_FILE,
];

fn main() {
    for pattern in INPUTS {
        println!("cargo:rerun-if-changed={}", pattern);
    }

    protoc_rust::Codegen::new()
        .out_dir("src")
        .inputs(&[PROTO_FILE])
        .include("input")
        .customize(protoc_rust::Customize {
            serde_derive: Some(true),
            .. Default::default()
        })
        .run()
        .expect("protoc");

    // TODO: Do I need to place results here?
    // use std::env;
    // let out_dir = env::var("OUT_DIR").unwrap();
    // println!("cargo:warning=OUT_DIR={}", out_dir);

    // TODO: Build web-client first? I guess I've been manually doing this so far.
}