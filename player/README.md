# Zest Player

By default, this app assumes that Zest is running on `0.0.0.0:6969` on the same machine, and that the browser is capable of decoding `audio/mpeg` using `MediaSource`.

Currently, the it relies on `SourceBuffer` to store audio in memory, which limits the maximum size of a single track to approximately ~10mb.

The default address can be edited in [main.js](./main.js).

# Quick start

Start Zest on default port:
```console
$ cargo run --release -- serve ...
```

Start a python web-server to serve this folder:
```console
$ python3 -m http.server <port>
```

The page will be accessible to everyone who has access to the specified port, so they can enjoy your music too :3c
