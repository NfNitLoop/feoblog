module.exports = {
    exclude: [
        // Seems odd that these aren't excluded by default:
        "package*.json",
        "snowpack.config.js",
        ".gitignore",

        // But also sometimes I `cargo run` in the wrong dir:
        "**/*.@(sqlite|sqlite3)",

    ],
    installOptions: {
        // bs58 -> safe-buffers -> buffer, needs polyfill:
        polyfillNode: true
    },
}