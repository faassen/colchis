# Colchis

Colchis is a Rust library that supports loading huge JSON data into memory. It
typically takes less storage than the original file, while still offering full
access to the JSON data.

It also uses internal data structures that should make particular search
operations very fast, but we haven't exposed them yet.

This library is highly experimental and in an early stage. It MAY eventually
become useful if you need to do a lot of in-memory queries on a massive
multi-gigabyte JSON file, and you don't mind waiting a while for the initial
parse, but we aren't there yet.

Colchis does this by using [succinct data
structures](https://blog.startifact.com/posts/succinct/): in particular a
balanced parenthesis tree to store tree structure information and Elias Fano
coded sparse bitvectors to store node type information.

During parsing we try to take care we don't go much above the size of the
original JSON file. We do this by compressing integer node type position
information using bitpacking. It needs to be uncompressed in the end to create
sparse bitvectors, but this can be done per node type, so this shouldn't
increase peak memory too much.

## Saving memory

Colchis has two distinct phases:

* parse time when a JSON file is loaded into memory

* runtime when the JSON file is ready for random access in memory

During parse time Colchis uses compression techniques to avoid high peak memory usage.

To prepare for runtime random access however, Colchis turns this compressed data
into succinct data structures. These take more memory than the compressed version,
but are still compact, and allow fast random access at the same time.

Care needs to be taken to avoid high peak memory usage throughout the process of parsing and preparing for runtime.

## Why this name?

[Jason](https://en.wikipedia.org/wiki/Jason), Greek hero of the
[Argonautica](https://en.wikipedia.org/wiki/Argonautica), went to
[Colchis](https://en.wikipedia.org/wiki/Colchis) at the eastern end of the world
in a quest for the Golden Fleece.

And JSON sounds like Jason.

## Credits

Paligo let me create [Xoz](https://github.com/Paligo/xoz), which uses the same
approach for XML. Many of the ideas are based on the paper [Fast in-memory XPath
search using compressed
indexes](https://repositorio.uchile.cl/bitstream/handle/2250/133086/Fast-in-memory-XPath-search-using-compressed-indexes.pdf).

Oh and to [Itamar](https://hachyderm.io/@itamarst) who accidentally [nerd-sniped](https://xkcd.com/356/) me.
