module.exports = {
    exclude: [
        // Seems odd that these aren't excluded by default:
        "package*.json",
        "snowpack.config.js",
        "svelte.config.js",
        "tsconfig.json",
        "**/.gitignore",

        // But also sometimes I `cargo run` in the wrong dir:
        "**/*.@(sqlite|sqlite3)",

    ],

    packageOptions: {
        // bs58 -> safe-buffers -> buffer, needs polyfill:
        polyfillNode: true,
        
        rollup: {
            plugins: [
                require('rollup-plugin-copy')({
                    targets: [
                        // Copy an unmodified version of this so it'll work in a web worker:
                        // (Snowpack likes to module-ify things, and most browsers don't support modules in web workers.)
                        {
                            src: "node_modules/tweetnacl/nacl-fast.min.js",
                            dest: "build/ts/naclWorker/",
                            rename: "tweetnacl.js"
                        },
                    ]
                })
            ]
        }
    },
    plugins: [
        // TODO: https://www.npmjs.com/package/snowpack-plugin-hash looks nice.

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
        ],
    ],
    buildOptions: {
        clean: true,
    },
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