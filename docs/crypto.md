Cryptography in FeoBlog
=======================

Basically, it uses [NaCl] signing keys and detached signatures, which are stored
as bytes (when inside [CBOR] data structures]) or displayed to the user [Base58]-encoded.

[NaCl]: https://en.wikipedia.org/wiki/NaCl_(software)
[CBOR]: https://en.wikipedia.org/wiki/CBOR
[Base58]: https://en.wikipedia.org/wiki/Base58