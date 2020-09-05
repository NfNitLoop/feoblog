module.exports = {
    exclude: [
        // Seems odd that these aren't excluded by default:
        "package*.json",
        "snowpack.config.js",
        "**/.gitignore",

        // But also sometimes I `cargo run` in the wrong dir:
        "**/*.@(sqlite|sqlite3)",

    ],

    installOptions: {
        // bs58 -> safe-buffers -> buffer, needs polyfill:
        polyfillNode: true
    },
    plugins: [
        // Use protoc-gen-ts to compile .proto files:
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
        
        // Run svelte-check for Typescript checking
        // See: https://github.com/pikapkg/snowpack/blob/master/create-snowpack-app/app-template-svelte-typescript/snowpack.config.json
        // When used with `snowpack build --wait`, only checks at startup.
        [
            "@snowpack/plugin-run-script",
            {
                "cmd": "svelte-check --output human",
                "watch": "$1 --watch",
                "output": "stream"
            }
        ]
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