pub mod http;

pub type Response = String;

pub fn handle(request: &http::Request) -> Response {
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
