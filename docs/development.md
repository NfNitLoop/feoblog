Local Development
===========

Implementation Details
----------------------

Some basic concepts to get you familiar with how things are organized:

* [URL Layout]
* [Data Format]
* [Cryptography]

[URL Layout]: ./docs/url_layout.md
[Data Format]: ./docs/data_format.md
[Cryptography]: ./docs/crypto.md

The UI for Feoblog is written using [TypeScript] inside of [Svelte] components, so familiarity with these technologies is useful.

[TypeScript]: https://typescriptlang.org/
[Svelte]: https://svelte.dev/

The server-side is implemented using [ActixWeb] and [Askama] templates.

[ActixWeb]: https://actix.rs/
[Askama]: https://github.com/djc/askama


Initial Setup
-------------

Install these dependencies:
 * [Rust]
 * [npm]
 * [protoc]

[rust]: https://rustup.rs/
[npm]: https://www.npmjs.com/get-npm
[protoc]: https://developers.google.com/protocol-buffers/

Run `npm install` inside the `web-client/` subdirectory to install the Javascript.

Run `cargo build` in the project root to build the web server portion.

Run `cargo run db init` to initialize an empty database for development.

Development Workflow
--------------------

I find that much of the work I do is based in the TypeScript/Svelte-based user
interface (inside the `web-client/` subdirectory). When running in development
mode, FeoBlog is configured to pass through access to those JavaScript files to
the underlying file system.  (vs. accessing static files that are bundled into
the binary when built in release mode.) This means that you don't have to
rebuild and relaunch the Rust-based web server every time you modify those
files.

To run in this mode, I recommend starting two terminal windows. 

1. In the first window, in the root of the project, start the web server:

       cargo run serve --open

2. In the second window, in the `web-client/` subdirectory, run:

       npm run watch

   This will start a process that will compile the Svelte/TypeScript code, and
   continually watch for changes and update the output as necessary.

Unfortunately, the Svelte compiler doesn't always pick up every change, so
depending on changes you make, you may need to stop the `npm run watch` process
and restart it. (This process can also periodically die, so if your changes
aren't showing up, check to make sure it's still running!)

Local Testing With SSL
----------------------

Feoblog itself does not currently support serving encrypted HTTP connections.
However, some functionality may require an SSL connection to work. In
particular, browser implementations of the [MediaStream] API may require an SSL
connection. (Though Chrome and Firefox seem to allow access to 127.0.0.1 without
it.) If you need to, you can run an SSL proxy like this:

[MediaStream]: https://developer.mozilla.org/en-US/docs/Web/API/MediaStream

```
# Start feoblog on a public IP address:
# (allows testing from other computers. Say, Safari on your iPhone)
cargo run serve --bind 192.168.1.101:8080

# Proxy to that port:
web-client/node_modules/.bin/local-ssl-proxy --hostname 192.168.1.101 --source 8081 --target 8080
```

This is not a great way to serve your production site, but it'll work easily
enough for development. Of course, it's using a self-signed certificate, so
you'll need to accept that when you first connect via SSL.


Building a Release
------------------

To build a "release"/self-contained version of Feoblog:

 * Build the web client.  
   In `web-client/`, run:

       npm run build

 * Make a release build.  
   In the root directory, run:

       cargo build --release

   * or, alternatively: `cargo install --path . --locked`

Note, this currently does not work on Windows.
See: <https://github.com/NfNitLoop/feoblog/issues/16>

This process is also automated as github actions:
<https://github.com/NfNitLoop/feoblog/blob/develop/.github/workflows/build.yml>

