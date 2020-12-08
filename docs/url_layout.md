URL Layout
==========

FeoBlog operates on [REST] principles. Pieces of data are fetchable by URL with
HTTP GET, and can be updloaded with HTTP PUT. Standardizing the locations of those data files, and their formats, will let different implementations communicate
with each other.

[REST]: https://en.wikipedia.org/wiki/Representational_state_transfer


`/`
---

The root of the server may display any type of user interface the implementation
desires. It may be a stream of latest posts on the server, or of a single
user's posts, if the server is the home of a single user.

`/u/<userID>/`
------------

This endpoint should generally list a user's posts in reverse chronological
order (most recent posts first). Whether those posts are shown in-line or as
links to the full posts is up to the implementaiton.

You may also display information about a user, such as their preferred name(s),
number/size of posts, "home server", etc., either inline or as links.


`/u/<userID>/i/<signature>/`
------------------------

URLs of this format point to a single piece of content from a user. The server
should render it for viewing.

 * `userID` is the base58-encoded NaCL public key.
 * `signature` is the base58-encoded signature of the post.

Rendering may take different forms for different types of content. I expect the
common case will be rendering a [CommonMark] post, or a reply to someone else's
post. 

[CommonMark]: https://commonmark.org/

`/u/<userID>/i/<signature>/proto3`
--------------------------------

This endpoint should serve the binary Protobuf data for a single post by a user.
Other clients must be able to fetch this data so that they can verify the
signature over that data was indeed made by the given userID.

Other clients/servers may also send an HTTP PUT to this endpoint to upload new
data. The server may decide whether to accept or reject the data. If the
server accepts the data, it should always verify that it is valid data, 
and is signed by the `userID` and `signature` provided in the URL.

`/u/<userID>/i/<signature>/files/*`
------------------------------

Some post types may allow the user to attach files. For example, a blog post
may contain photos which the user wants to display inline.

Any files with user-specified names will be served from this directory, to
distinguish them from files with standard names.

User-specified file names must be in UTF-8, must not start with a `.`, and must
not contain a `/`.  i.e.: You can not create or simulate nested file paths
within the `files/` URL.

Clients/servers may PUT files to these locations after the raw Protobuf data has
been published (at `/<userID>/<signature>/proto3`). The server must verify
that the posted data matches the corresponding hash and size as specified in the
Protobuf data. (TODO: Not yet implemented.)
