FeoBlog
=======

FeoBlog is a distributed blogging platform. It takes a lot of its
inspiration from [Mastodon] and [Scuttlebutt]. It aims to solve a couple of
problems with those services.

[Mastodon]: https://joinmastodon.org/
[Scuttlebutt]: https://www.scuttlebutt.nz/

On Mastodon, your user identity is tied to a particular server. If that server
goes down, your content, your user ID and your social network disappear with it.
But with FeoBlog, your ID is yours. Your content can be hosted on multiple
servers, so that if one of them goes down, your content isn't all lost. If you
want to leave a server, you can just move your content to another server without
getting a new user ID and having to manually tell all of your friends.

Scuttlebutt tries to be a distributed system, but it has one big problem: Only
one device can post new content. If you copy your identity to two devices and
post from both of them, you can get into a state where your identity is broken.
This is because Scuttlebutt tries to maintain an append-only, linear chain of
history (a bit like a blockchain) which could fork if you use it on devices.
(And Scuttlebutt doesn't deal well with forks.)
FeoBlog treats your identity as a *collection* of posts by you. (Not an
append-only chain.) This lets you post from multiple devices, and lets servers
eventually fetch and include all posts from all of those devices. This also
lets servers and users delete some of your content, for example, if they only
want to host the most recent posts by you, or want to exclude content that
uses too much disk space.

OK, well, all of this is theory so far. These are the grand plans. Quotas
aren't implemented yet.

Core Features
-------------

The main feature of FeoBlog is its data structure. Anyone can create a
[NaCL][Cryptography] public key, and use it (base58-encoded) as a user ID. A
"blog" for a user is just a collection if binary protobuf objects which are
signed by that user. Each object must be an `Item`, which is defined in
[feoblog.proto].

Methods for storing and displaying these items can be developed separately by different
clients. This repository includes two renderers for such content.  One is plain
HTML, rendered server-side, accessed at paths like `/u/:userID/i/:signature/`.
The other is an in-browser JavaScript client which fetches the binary data
directly from the server and renders it client-side, at a URL like
`/client/#/u/:userID/i/:signature`.

Because items are cryptographically signed by the user who posts them, they can
be safely copied around to different servers and caches by anyone. And anyone
can read them knowing that they haven't been modified.

For servers/clients on the web, there are [standard URLs][URL Layout] for getting and
sending `Item`s.

### Other featuers ###

 * Uses a safe subset of CommonMark markdown for posts.
 * Can easily run a server locally
   * Sync content from those you follow to have offline.
   * Compose posts offline, and send them all when you're back online.

### Planned features ###

 * Comments
 * "Reply" posts which link to the Item they reply to
 * File attachments
 * Revoking user IDs (i.e.: "Delete my account.")

 ### Unplaned features ###

There are certain features that I do not plan to implment, because I think they
are deterimental in social networks.

 * Likes, or counts for likes, replies, or follows. These are easy to game and
   people assign too much meaning to them.
 * Edits or deletes. Content you post is crytpographically signed and visible
   forever, unless you revoke your userID.
 * Reblogging. I belive there should be a higher barrier to sharing others'
   content. You'll need to comment, post, or "Reply" post to share content to
   your followers.


[feoblog.proto]: ./protobufs/feoblog.proto

The Name
--------

I'm not a great UI designer, so my blog will be a bit [feo]. Fe2O3 is also
the chemical [formula] for rust, and this implementation is written in [Rust]. :p 

[feo]: https://en.wiktionary.org/wiki/feo#Spanish
[formula]: https://en.wikipedia.org/wiki/Iron(III)_oxide
[Rust]: https://www.rust-lang.org/

Implementation Details
======================

* [URL Layout]
* [Data Format]
* [Cryptography]

[URL Layout]: ./docs/url_layout.md
[Data Format]: ./docs/data_format.md
[Cryptography]: ./docs/crypto.md


Development
===========

Dependencies:
 * npm
 * protoc

You may need to run `npm install` inside the `web-client` subdirectory.

To develop the interactive web client, run `npm run watch` in the `web-client`
subdirectory, then (in another window) run `cargo run serve --open`.

Building
========

To build a "release", self-contained version of Feoblog:

* In `web-client/`, run `npm run build`
* In the root directory, run `cargo build --release`
  * or, alternatively: `cargo install --path . --locked`