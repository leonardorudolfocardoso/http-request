use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

type Request = String;
type Response = String;

fn handle(request: &Request) -> Response {
    match request.trim() {
        "GET /test HTTP/1.1" => {
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

fn read<R: BufRead>(r: &mut R) -> Request {
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
                let request = read(&mut buf);
                let response = handle(&request);

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
