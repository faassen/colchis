# Colchis

Colchis is a Rust library that supports loading huge JSON data into memory. It
typically takes less storage than the original file, while still offering full
access to the JSON data.

It also uses internal data structures that should make particular search
operations very fast, but we haven't exposed them yet.

This library is highly experimental.

Colchis does this by using succinct data structures: in particular a balanced
parenthesis tree to store tree structure information and Elias Fano coded sparse
bitvectors to store node type information.

During parsing we try to take care we don't go much above the size of the
original JSON file. We do this by bitpacking to compress integer node type
position information. It needs to be uncompressed in the end to create sparse
bitvectors, but this can be done per node type, so this shouldn't increase peak
memory too much.

## Why this name?

[Jason](https://en.wikipedia.org/wiki/Jason), Greek hero of the
[Argonautica](https://en.wikipedia.org/wiki/Argonautica), went to
[Colchis](https://en.wikipedia.org/wiki/Colchis) at the eastern end of the world
in a quest for the Golden Fleece.

And JSON sounds like Jason.
