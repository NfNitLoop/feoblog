Data Format
===========

FeoBlog uses base58-encoded [NaCL] public keys to identify users, and
base58-encoded signatures to identify "posts" by users.

The signatures sign a binary [CBOR] payload that includes metata about the post.

[NaCL]: https://nacl.cr.yp.to/
[CBOR]: https://cbor.io/

Required fields
---------------

Every post must include the following fields:

 * `utcTime`: a signed 64-bit integer representing unix time in the UTC time zone.
 * `type`: A string representing the type of the post.

   Valid types:
    * `post`: a markdown post 


Optional Fields
---------------
 * `utcOffset`: A string of the format (+|-)hhmm (ex: -0700) representing the
   post's time zone relative to UTC. This may be used to display the post's
   timestamp using local time.


Type: "post"
============

A post reprents a blog post made in CommonMark (Markdown) format. A post has the following additional attributes:

Required:

 * `content`: bytes representing the multihash of the blog post.

Optional:

 * `title`: A title for the post.
 * `files`: a list 
