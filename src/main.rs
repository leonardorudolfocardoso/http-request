use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

type RawRequest = Vec<u8>;
type Headers<'a> = HashMap<&'a str, &'a str>;

struct Request<'a> {
    version: &'a str,
    method: &'a str,
    path: &'a str,
    headers: Headers<'a>,
}
type Response = String;

fn handle(request: &Request) -> Response {
    match (request.method, request.path, request.version) {
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

#[derive(Debug)]
struct InvalidRequest;

fn parse<'a>(raw: &'a RawRequest) -> Result<Request<'a>, InvalidRequest> {
    let request_line_end = raw
        .windows(2)
        .position(|bytes| bytes == b"\r\n")
        .ok_or(InvalidRequest)?;
    let request_line_bytes = &raw[..request_line_end];
    let request_line = str::from_utf8(request_line_bytes).unwrap();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap();
    let path = parts.next().unwrap();
    let version = parts.next().unwrap();

    let header_start = request_line_end + 2;
    let header_end = header_start
        + raw[header_start..]
            .windows(4)
            .position(|bytes| bytes == b"\r\n\r\n")
            .unwrap();

    let header_bytes = &raw[header_start..header_end];
    let headers_text = str::from_utf8(header_bytes).unwrap();

    let mut headers = Headers::new();

    for line in headers_text.split("\r\n") {
        let (key, value) = line.split_once(": ").unwrap();
        headers.insert(key, value);
    }

    Ok(Request {
        version,
        method,
        path,
        headers,
    })
}

fn read_request<R: BufRead>(reader: &mut R, buf: &mut RawRequest) {
    loop {
        let mut line = Vec::new();

        let n = reader.read_until(b'\n', &mut line).unwrap();

        if n == 0 {
            break;
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
    reader.read_exact(&mut buf[start..]).unwrap();
}

fn should_close(request: &Request) -> bool {
    request.headers.iter().any(|(key, value)| {
        key.eq_ignore_ascii_case("connection") && value.eq_ignore_ascii_case("close")
    })
}

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut reader = BufReader::new(&stream);
                let mut raw = RawRequest::with_capacity(4096);
                loop {
                    raw.clear();

                    read_request(&mut reader, &mut raw);

                    if raw.is_empty() {
                        break;
                    }

                    match parse(&raw) {
                        Ok(request) => {
                            let response = handle(&request);

                            reader.get_mut().write_all(response.as_bytes()).unwrap();

                            if should_close(&request) {
                                break;
                            }
                        }
                        Err(e) => eprintln!("{e:?}"),
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
