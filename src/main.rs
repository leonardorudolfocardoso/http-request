use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn handle(request: &str) -> String {
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

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = BufReader::new(&stream);
                let mut request = Vec::new();
                loop {
                    let mut line = String::new();
                    let bytes = buf.read_line(&mut line).unwrap();

                    if bytes == 0 {
                        break;
                    }

                    if line == "\r\n" {
                        break;
                    }

                    request.push(line);
                }

                let req = request.first().unwrap();

                let response = handle(req);

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
