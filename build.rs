
// use protoc_rust;

fn main() {
    // TODO: Specify a rebuild-if
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protobufs/feoblog.proto"])
        .include("protobufs")
        .run()
        .expect("protoc");

    // TODO: Do I need to place results here?
    // use std::env;
    // let out_dir = env::var("OUT_DIR").unwrap();
    // println!("cargo:warning=OUT_DIR={}", out_dir);
}