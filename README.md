# http-server

A small Rust HTTP/1.1 server used to exercise request parsing, response formatting, and connection handling.

The binary listens on `127.0.0.1:8080` and serves each accepted TCP connection on a Tokio current-thread runtime.

## Running

```sh
cargo run
```

Example request:

```sh
curl http://127.0.0.1:8080/test
```

## Routes

| Method | Path | Response |
| --- | --- | --- |
| `GET` | `/test` | `200 OK` with `Hello world!` |
| `GET` | `/sleep` | waits 5 seconds, then returns `200 OK` with `Slept for 5s` |
| any other parsed request | any path | `404 NOT FOUND` |

Malformed requests return `400 BAD REQUEST`, include `Connection: close`, and stop processing that connection.

## Connection Behavior

The server can process multiple requests from the same connection. It keeps reading until one of these happens:

- the client closes the connection
- a request includes `Connection: close`
- request parsing or reading fails

Bodies are read when a `Content-Length` header is present. Header lines currently must use `Name: value` formatting and CRLF line endings.

## Testing

```sh
cargo test
```

The test suite covers request parsing, response formatting, synchronous connection handling, and asynchronous connection handling.

## Current Limitations

This is intentionally minimal and does not yet implement the full HTTP specification. Notable gaps include chunked transfer encoding, robust header normalization, configurable bind address, graceful shutdown, and application routing beyond the hard-coded examples.
