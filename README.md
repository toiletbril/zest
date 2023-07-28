# Zest

Web-server with main goal of streaming music in small chunks from all `mp3` files of any bitrate from a folder and it's subfolders. Additional features will be decided at a later stage.

Initially built with no dependencies (**and possibly not up to any of HTTP/web security standards**), it is currently usable, althrough highly unstable.

## Limitations
- Currently, only `mp3` files are supported.
- As SSL or authorization has not yet been planned, Zest will need to be paired with a reverse proxy that supports necessary features for real-world backend usage.

## Player

Zest includes a [default player/frontend](./player/).

## API

API documentation can be viewed [here](./API.md).

## Building

Clone the project and enter it's directory:
```console
$ git clone https://github.com/toiletbril/zest.git
$ cd zest
```

Build it with cargo:
```console
$ cargo build --release
```

Executable file will be available in `target/release/zest`.

You can also run the executable directly with:
```console
$ cargo run --release -- <...>
```

## Running

First, you will need to index a music directory. For example:
```console
$ zest index /run/media/music
Running Zest web-server, version 0.4.0-unstable (c) toiletbril <https://github.com/toiletbril>
Traversing '/run/media/music'...
Successfully traversed '/run/media/music', generated index file './zest-index-0.json'.
```

Then you can run Zest by serving the generated index:
```console
$ zest serve zest-index-0.json -p 1234 -t 16 -l -u 3
Running Zest web-server, version 0.4.0-unstable (c) toiletbril <https://github.com/toiletbril>
1 [18:13:16] ThreadId(1) -> MAIN: Starting the dispatcher...
2 [18:13:16] ThreadId(1) -> MAIN: Starting the logger...
3 [18:13:16] ThreadId(2) -> DISPATCHER: Binding to <http://localhost:1234>...
4 [18:13:16] ThreadId(2) -> DISPATCHER: Started. Available threads: 16.
```
