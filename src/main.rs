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

fn parse<'a>(raw: &'a RawRequest) -> Request<'a> {
    let request_line_end = raw.windows(2).position(|bytes| bytes == b"\r\n").unwrap();
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

    Request {
        version,
        method,
        path,
        headers,
    }
}

fn read<R: BufRead>(r: &mut R) -> RawRequest {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).unwrap();
    buf
}

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = BufReader::new(&stream);
                let raw = read(&mut buf);
                let request = parse(&raw);
                let response = handle(&request);

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
