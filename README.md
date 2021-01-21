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
 * [Rust]
 * [npm]
 * [protoc]

[npm]: https://www.npmjs.com/get-npm
[protoc]: https://developers.google.com/protocol-buffers/

You may need to run `npm install` inside the `web-client` subdirectory.

To develop the interactive web client, run `npm run watch` in the `web-client`
subdirectory, then (in another window) run `cargo run serve --open`.

Building
========

To build a "release"/self-contained version of Feoblog:

* In `web-client/`, run `npm run build`
* In the root directory, run `cargo build --release`
  * or, alternatively: `cargo install --path . --locked`


Getting Started
===============

Run the server
--------------

Once you've built or downloaded feoblog, you can run it locally by just running:

```
feoblog serve --open
```

This will:
 * Start a server on localhost:8080. (You can override w/ the `--bind` option)
 * Create a database called feoblog.sqlite3 in the current directory.
 * Open a web browser window pointing to your new empty database.

Create a User ID
----------------

By default, `http://localhost:8080/` will show you the plain HTML version of the site. This is a version of the site that's indexable by search engines, and readable by software and browsers that don't have JavaScript available.

To do more than just read existing content, you'll need to use a client. FeoBlog comes with a built-in client that runs in your browser.

Click the "Client" link to open the in-browser client.

Next, click the "Log in" link.

At the bottom of the page, click the "Create new ID button". This will generate a new user ID for you. It'll look like a random string of characters. For example: `A719rvsCkuN2SC5W2vz5hypDE2SpevNTUsEXrVFe9XQ7`.

The page will also generate a password for you. It's important to **save this key in a secure location** like a [password manager]. You can't change or reset this password. (It's a cryptographic private key that corresponds to your public ID.) 

[password manager]: https://en.wikipedia.org/wiki/Password_manager

Add Yourself to the Server
--------------------------

Now that you've generated a userID for yourself, you need to tell the server who you are.

A FeoBlog server doesn't contain any passwords, all it knows is a list of user IDs that are allowed to post content to it. Since all content is cryptographically signed by a user, the server can verify that a post came from you without your password.

So, using the userID above, to add myself to the server I'd just run:

```
feoblog user add A719rvsCkuN2SC5W2vz5hypDE2SpevNTUsEXrVFe9XQ7 --on-homepage --comment "Official FeoBlog Blob
```

You can do this by stopping the server with Ctrl-C first, or by running the command in a new window. (But make sure to re-start the server before the next steps!)

The optional `--on-homepage` argument says that posts you post to this ID should appear on the Home page of the feoblog, as well as in your individual user page.

And the optional `--comment X` argument is just a comment to help you, the server admin, keep track of who that ID is. It's only ever shown in the output of `feoblog user list`.

Log In
------

Return to the "Log In" page in the browser.

Paste your userID (not password!) into the "Log In As:" field, and click the "Log In" button. You'll get a warning that your profile doesn't exist. (We'll write one next!) Click the "Confirm" button.

Now you're "logged in". You may be surprised that no password was required. "Logging in" to the client just tells it to present data to you as if you are that user. Don't worry, a password will be required to write any data.

The client is built with the idea that you may manage multiple identities. If you generate another ID, you can "log in" as that identity as well, and the client will remember so that you can easily switch between them.

To help distinguish your identities, you can (and should!) give them different names and colors. Just edit the "Name" and "Color" fields as you wish. Changes are saved immediately. Colors should be 3- or 6-digit hexadecimal colors like `#03c` or `#0033cc`. Here's a handy [color picker] for you.

[color picker]: https://www.w3schools.com/colors/colors_picker.asp

Create Your Profile
-------------------

User IDs are not a great way to remember people. Thankfully FeoBlog supports user profiles. There, you can set a name for yourself and provide a short description of yourself and/or the purpose of your blog.

Click on the "My Profile" link.

FeoBlog again warns you that it can't find an existing profile for you. If a profile does exist, you have the option to "Sync from another server" so that you can re-use (or modify) the existing one. But since we're creating a new ID, click "Create New Profile".

For now, the important parts to set are:

 * Profile Display Name -- this is a friendly name or nickname to display instead of your long randomly-generated userID. You *should* set this, but it's not required, if you really prefer being an anonymous number.
 * The "Your profile here..." text box serves as a description for you and/or your blog. Write whatever you want. This box accepts "CommonMark" Markdown formatting.

Once you've got those set, now paste your password into the "Private Key" field, and click "Sign".

This will automatically generate a cryptographic signature for this content. If everything still looks good, click the "Send" button to send this profile to the server.

Write Your First Post
---------------------

Click on the "New Post" link at the top/left of the page.

This interface is very much like the "My Profile" page. Fill out the title and body of your post, then sign and send the post.

You can view it in "My Feed" and (if you enabled `--on-homepage` above) on the "Home" feed.

Linking
-------

Since FeoBlog content is distributed, and may be hosted on multiple servers, you should avoid hard-linking to a particular server. If you want to link to a FeoBlog userID or post within your post, use relative links, like this:

```
Did you see the [FeoBlog] [first post]?

[FeoBlog]: /u/A719rvsCkuN2SC5W2vz5hypDE2SpevNTUsEXrVFe9XQ7/
[first post]: /u/A719rvsCkuN2SC5W2vz5hypDE2SpevNTUsEXrVFe9XQ7/i/2F6NB6PYKDTPGTc9dfaQHpmPzd3LSjVgBuC6qa2hcLUJA74LbZpV8wL5HoXDmvzyfZWaX6sLyg3DoGtqh3t2rJt5/
```

Of course, if you're linking someone *outside* of FeoBlog to a partticular post, you can link them directly to a page on a particular server like this:

`https://blog.nfnitloop.com/u/A719rvsCkuN2SC5W2vz5hypDE2SpevNTUsEXrVFe9XQ7/i/2F6NB6PYKDTPGTc9dfaQHpmPzd3LSjVgBuC6qa2hcLUJA74LbZpV8wL5HoXDmvzyfZWaX6sLyg3DoGtqh3t2rJt5/`

The URLs are a bit long, but many services (like Twitter) will shorten them for you anyway. Plus, the URL contains a globally unique ID which can also be used to cryptographically verify the contents of the post. If any one server goes down, the `/u/...` relative path can be used on any other FeoBlog server that contains a copy of that item.

Advanced Topics
---------------

I should probably write more about these things? Tell me if you'd find them useful.

* Using Sync to copy your content between servers. (Hopefully the in-client info is enough for now?)
* Running a server behind Apache
* Running a server in Docker
* Writing your own client 
* Writing your own server