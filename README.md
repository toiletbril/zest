# Zest

Web-server with main goal of streaming music in small chunks from all `mp3` files of any bitrate from a folder and it's subfolders. The rest will be decided later.

Initially built with no dependencies (and possibly not up to any of HTTP standarts).

Concatenating and playing music chunks is left up to you.

## Limitations
- Only `mp3` files are supported for now.
- As no SSL or authorization is planned yet, for real-world backend usage you will need to pair Zest with reverse proxy that supports everything you need.

## Player

Zest comes with a [default player/frontend](./player/).

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
