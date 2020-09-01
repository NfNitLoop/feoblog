module.exports = {
    exclude: [
        // Seems odd that these aren't excluded by default:
        "package*.json",
        "snowpack.config.js",
        "**/.gitignore",

        // This is an intermediate build file, later processed by browserify.
        "protos/feoblog.proto3_pb.js",

        // But also sometimes I `cargo run` in the wrong dir:
        "**/*.@(sqlite|sqlite3)",

    ],

    installOptions: {
        // bs58 -> safe-buffers -> buffer, needs polyfill:
        polyfillNode: true
    },
    plugins: [
        [
            "@snowpack/plugin-run-script",
            {
                cmd: (
                    "protoc"
                    + " --js_out=import_style=commonjs,binary:protos"
                    // + " --js_out=library=protos,binary:protos"
                    + " --proto_path=../protobufs/"
                    + " feoblog.proto3"
                ),
                // TODO: watch?
            },
        ],
        [
            "@snowpack/plugin-run-script",
            {
                cmd: "browserify ./protos/feoblog.proto3_pb.js -o ./protos/feoblog.js",
            },
        ],
    ],
    buildOptions: {
        clean: true,
    }
}