
// use protoc_rust;

const PROTO_FILE: &str = "protobufs/feoblog.proto";

// Build will be re-run if any of these have changed:
const INPUTS: [&str; 4] = [
    "build.rs",
    PROTO_FILE,
    
    // Directories are checked recursively:
    // see: https://github.com/rust-lang/cargo/commit/cee088b0db01076deb11c037fe8b64b238b005a2
    "static/",
    "web-client/build/",
];

fn main() {
    for pattern in INPUTS {
        println!("cargo:rerun-if-changed={}", pattern);
    }

    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&[PROTO_FILE])
        .include("protobufs")
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