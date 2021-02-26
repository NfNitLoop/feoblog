Changelog
=========

Version 0.3.0
=============

Released: Feb. 25, 2021  
<https://github.com/NfNitLoop/feoblog/releases/tag/v0.3.0>

New Features
------------

 * Attachments!  
   You can now attach files to posts by dragging them onto the post editor. A
   link to the file will be automatically generated for you. If the file is an
   image, it'll be inlined in your post by default. Syncing between servers will
   also sync file attachments.
 * Release automation.  
   This is more for me than for y'all, but the result of this is that releases should be regularly available via the [releases] page.

[releases]: https://github.com/NfNitLoop/feoblog/releases

Note: There's a known issue ([Bug #16]) that is preventing Windows builds from working at the moment. I'll enable Windows builds when that's fixed.

[Bug #16]: https://github.com/NfNitLoop/feoblog/issues/16


Improvements
------------

 * Switched to comrak for server-side (plain HTML) rendering of Markdown.  
   Users shouldn't notice any changes, but this library operates in a "safe by
   default" mode which is nice.
 * Improved browser caching.   
   Protobuf "Items" (and file attachments) are now served with HTTP headers to
   allow browsers to cache them indefinitely, since they should never change.
 * SQLite's "Write Ahead Logging" ("WAL") mode is now enabled when available,
   which greatly increases write throughput when syncing. This also means that
   reads and writes do not block each other.
 * Disable in-browser signature verification during sync.  
   This further improves sync speed, since in-browser crypto is particularly slow.
   The server will still validate that the objects it receives are cryptographically
   signed. (And the in-browser client still always verifies content signatures before
   displaying them.)
  

Bug Fixes
---------

 * Fixed some minor rendering issues when viewing a server w/ no posts.

Version 0.2.2
=============

Released: Feb. 5, 2021

Bug Fixes
---------

 * You need not be logged in to view comments in the client.

Version 0.2.1
=============

Released: Feb. 5, 2021

Bug Fixes
---------

* Fix a broken build.  
  Building in dev mode doesn't necessarily expose all the broken parts. Oops!


Version 0.2.0
=============

Released: Feb. 5, 2021

New Features
------------

 * Comments!
    * You can leave comments on any post or comment.
    * Comments only show up in your feed in the client, not on the plain HTML
      version of the site, or on the home page.
 * Lots of style updates. Items (post/comments) now have a single-line header of
   metadata.
 * [Identicons]!  
   For users that don't have a profile photo (which is everyone, at the moment),
   you'll get a default identicon based on your user ID. This makes it easy to
   quickly see who posts are from in your feed.
 * Server-side support for replies.
   * List all replies to a given Item via `/u/:userID/i/:signature/replies/proto3`
   * Index reply Items so it's efficient to look them up.
 * New `feoblog db upgrade` command to keep your database up-to-date with the
   latest versions of FeoBlog.
 * Relative timestamps. (But you can mouse-over a date to see absolute times.)

[Identicons]: https://en.wikipedia.org/wiki/Identicon


Improvements
------------

 * Much faster loading on endless-scrolling pages of items.
 * Posting a new Item will take you to its page after upload.
 * Much simplified Sync process.  
   The "bootstrap" step from v0.1.0 is no longer necessary.
 * Improved sync progress logging.  
   Click on subsections to show/hide as much detail as you care to see.
 * Explicit `feoblog db init` command.  
   This will require an extra step during initial setup, but it means you can't
   accidentally create new databases.
 * Refactored database code so that it should be even easier to replace w/
   alternate implementations if/when they exist.
 * Add the `<meta ...>` tag to make iOS devices happy.  
   I still haven't done a whole lot of testing on these devices, but the scaling
   is now much better at least.

Bug Fixes
---------

 * Disallow uploading Items dated in the future.  
   This was likely due to user error. The server doesn't show future items, so it
   also resulted in unexpected behavior.

Version 0.1.0
=============

Released: Jan. 20, 2021

New Features
------------

 * First release!
 * Posts
 * Profiles
 * Syncing between servers
 * Plain HTML "Web 1.0" version
 * In-browser JavaScript client.
