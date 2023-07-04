# Example player

This example assumes that Zest is running on `http://localhost:6969` and your browser supports decoding of `audio/mpeg` with `MediaSource`.
It also relies on `SourceBuffer` to store audio in memory, which means that max size for one track is ~10mb.

# Quick start

Start Zest on default port:
```console
$ cargo run --release -- serve ...
```

Start a python web-server to serve this folder:
```console
$ python3 -m http.server <port>
```

This page will be available to everyone who has access to that port, so they can enjoy your music too :3c