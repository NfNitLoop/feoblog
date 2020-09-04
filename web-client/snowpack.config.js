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
                    + ` --plugin=./node_modules/.bin/${npmScript('protoc-gen-ts')}`
                    + " --ts_out=protos"
                    + " --proto_path=../protobufs/"
                    + " feoblog.proto"
                ),
                // TODO: watch?
            },
        ],
        ["@snowpack/plugin-svelte"],
    ],
    buildOptions: {
        clean: true,
    }
}

function npmScript(name) {
    if (isWindows()) {
        return `${name}.cmd`
    }
    return name
}

function isWindows() {
    return process.platform == "win32"
}