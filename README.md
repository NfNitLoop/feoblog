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

OK, well, all of this is theory so far. These are the grand plans. These things
aren't implemented yet.

The Name
--------

I'm not a great UI designer, so my blog will be a bit [feo]. Fe2O3 is also
the chemical [formula] for rust, and this implementation is written in Rust. :p 

[feo]: https://en.wiktionary.org/wiki/feo#Spanish
[formula]: https://en.wikipedia.org/wiki/Iron(III)_oxide

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
subdirectory, then run `cargo run serve`.

