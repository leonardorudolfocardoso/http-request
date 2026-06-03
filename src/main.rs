use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

type RawRequest = String;
type Headers = HashMap<String, String>;

struct Request {
    version: String,
    method: String,
    path: String,
    headers: Headers,
}
type Response = String;

fn handle(request: &Request) -> Response {
    match (
        request.method.as_str(),
        request.path.as_str(),
        request.version.as_str(),
    ) {
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

fn parse(raw: RawRequest) -> Request {
    let mut lines = raw.lines();
    let request_line = lines.next().unwrap();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap().to_owned();
    let path = parts.next().unwrap().to_owned();
    let version = parts.next().unwrap().to_owned();

    let mut headers = Headers::new();

    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_owned(), value.to_owned());
        }
    }

    Request {
        version,
        method,
        path,
        headers,
    }
}

fn read<R: BufRead>(r: &mut R) -> RawRequest {
    let mut request = String::new();
    r.read_to_string(&mut request).unwrap();
    request
}

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = BufReader::new(&stream);
                let raw = read(&mut buf);
                let request = parse(raw);
                let response = handle(&request);

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
