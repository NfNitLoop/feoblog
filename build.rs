
// use protoc_rust;

fn main() {
    let proto_file = "protobufs/feoblog.proto";
    println!("cargo:rerun-if-changed={}", proto_file);
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&[proto_file])
        .include("protobufs")
        .run()
        .expect("protoc");

    // TODO: Do I need to place results here?
    // use std::env;
    // let out_dir = env::var("OUT_DIR").unwrap();
    // println!("cargo:warning=OUT_DIR={}", out_dir);

    // TODO: Build web-client first? I guess I've been manually doing this so far.
}