Protobufs
=========

FeoBlog uses proto3 files to define the data format for many data structures.
It provides a concise binary format which we can sign, as well as an extensible
way to define and extend schemas over time.

The protos/ subdirectory is used both to generate Rust bindings, but also
exposed via the web so that browser-based clients can discover them.