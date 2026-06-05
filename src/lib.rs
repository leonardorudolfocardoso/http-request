use std::io::BufRead;

use crate::http::{RawRequest, Request};

pub mod http;

pub type Response = String;

pub fn handle(request: &Request) -> Response {
    match (request.method(), request.path(), request.version()) {
        ("GET", "/test", "HTTP/1.1") => {
            let body = "Hello world!";

            format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Length: {}\r\n\
                 \r\n\
                  {}",
                body.len(),
                body
            )
        }
        _ => "HTTP/1.1 404 NOT FOUND\r\n\
              Content-Length: 0\r\n\
              \r\n\
                    "
        .to_owned(),
    }
}

pub enum ReadRequest {
    Complete,
    Closed,
}

pub fn read_request<R: BufRead>(
    reader: &mut R,
    buf: &mut RawRequest,
) -> std::io::Result<ReadRequest> {
    loop {
        let mut line = Vec::new();

        let n = reader.read_until(b'\n', &mut line)?;

        if n == 0 {
            return Ok(ReadRequest::Closed);
        }

        let end = line == b"\r\n";

        buf.extend(line);

        if end {
            break;
        }
    }

    let headers = std::str::from_utf8(buf).unwrap();

    let content_length = headers
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(": ")?;
            if key.eq_ignore_ascii_case("content-length") {
                value.parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let start = buf.len();

    buf.resize(start + content_length, 0);
    reader.read_exact(&mut buf[start..])?;

    Ok(ReadRequest::Complete)
}
