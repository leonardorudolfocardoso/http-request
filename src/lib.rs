use crate::http::{RawRequest, Request};

pub mod http;

pub type Response = String;

pub fn handle<'a>(request: &'a RawRequest) -> (Option<Request<'a>>, Response) {
    let request = Request::try_from(request);
    let response = match request {
        Ok(ref request) => match (request.method(), request.path(), request.version()) {
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
        },
        Err(ref e) => {
            eprintln!("{e:?}");
            let body = e.to_string();
            let len = body.len();

            format!(
                "HTTP/1.1 400 BAD REQUEST\r\n\
                Content-Length: {len}\r\n\
                \r\n\
                {body}"
            )
        }
    };

    (request.ok(), response)
}
