# Zest

Music-streaming web-server.

The goal is an ability to stream music in chunks from all music files from a folder and it's subfolders.

Initially built with no dependencies (and possibly not up to any of HTTP standarts). As I continue the development, dependencies will be listed in [`Cargo.toml`](./Cargo.toml).

## Limitations
- Only `mp3` is supported for now.
- As no SSL or authorization is planned, for real-world usage you will need to bundle Zest with reverse proxy that supports everything you need.

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

## Running the web-server

First, you will need to index a music directory. For example:
```console
$ zest index /run/media/music
Running Zest web-server, version 0.4.0-unstable (c) toiletbril <https://github.com/toiletbril>
Traversing '/run/media/music'...
Successfully traversed '/run/media/music', generated index file './zest-index-0.json'.
```

Then you can serve it:
```console
$ ./zest serve zest-index-0.json -p 1234 -t 16 -l -u 3
Running Zest web-server, version 0.4.0-unstable (c) toiletbril <https://github.com/toiletbril>
1 [18:13:16] ThreadId(1) -> MAIN: Starting the dispatcher...
2 [18:13:16] ThreadId(1) -> MAIN: Starting the logger...
3 [18:13:16] ThreadId(2) -> DISPATCHER: Binding to <http://localhost:1234>...
4 [18:13:16] ThreadId(2) -> DISPATCHER: Started. Available threads: 16.
```

### Help

```console
$ zest --help
USAGE: zest [-options] <subcommand>
Music-streaming web-server.

SUBCOMMANDS:  serve <index file>     	Serve the music.
              index <directory>      	Index directory and make an index file.

OPTIONS:      -p, --port <port>      	Set server's port.
              -a, --address <adress> 	Set server's address.
              -t, --threads <count>  	Threads to create.
              -u, --utc <hours>      	UTC adjustment for logger.
              -l, --log-file         	Create a log file.

0.4.0-unstable (c) toiletbril <https://github.com/toiletbril>
```

## Endpoints

All endpoints in this API are prefixed with `/api/v1/music`.

### Chunk of music file

Returns a specified chunk of a music file.

- Method: `GET`
- Endpoint: `/get`
- Parameters:
  - `name` (string, required): The name of the music track.
  - `chunk` (integer, default is 0): The index of the 512kb chunk of the music file.
- Response:
  - Content-Type: `audio/mpeg`
  - Body: The chunk of the specified music file.
- Errors:
  - `416 Requested Range Not Satisfiable`: When the specified `chunk` is out of range for the music file.
  - `400 Bad Request`: When the `name` parameter is not specified.

Example Request:
```http
GET /api/v1/music/get?name=HelloWorld&chunk=2 HTTP/1.1
Host: your-api-domain.com
```

Example Response:
```http
HTTP/1.1 200 OK
Content-Type: audio/mpeg

[Chunk of the specified music file]
```

### List of available track names
Returns a list of all available music track names.

- Method: `GET`
- Endpoint: `/all`
- Response:
    - Content-Type: application/json
    - Body: An array containing all available music track names.

Example response:
```http
HTTP/1.1 200 OK
Content-Type: application/json

[ "track1", "track2", "track3" ]
```
