# Zest API

All endpoints are prefixed with `/api/v1/music`.

### Chunk of music file

Returns a specified 128 kb chunk of a music file.

- Method: `GET`
- Endpoint: `/get`
- Parameters:
  - `name` (string, required): The name of the music track.
  - `chunk` (integer, default is 0): The index of the chunk.
- Response:
  - `Content-Type`: `audio/mpeg`
  - Body: The chunk of the specified music file.
- Errors:
  - `416 Requested Range Not Satisfiable`: When the specified `chunk` is out of range for the music file.
  - `400 Bad Request`: When the `name` parameter is not specified.

Example Request:
```http
GET /api/v1/music/get?name=HelloWorld&chunk=2 HTTP/1.1
Origin: some-domain.com
```

Example Response:
```http
HTTP/1.1 200 OK
Content-Type: audio/mpeg

<Chunk of the music file specified>
```

### List of available track names
Returns a list of all available music track names.

- Method: `GET`
- Endpoint: `/all`
- Response:
    - `Content-Type`: `application/json`
    - Body: An array containing all available music track names.

  Example Request:
```http
GET /api/v1/music/all HTTP/1.1
Origin: some-domain.com
```

Example response:
```http
HTTP/1.1 200 OK
Content-Type: application/json

[ "track1", "track2", "track3" ]
```