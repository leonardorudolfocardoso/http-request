use crate::http::{Headers, RawRequest, Request, Response};

pub mod http;

pub fn handle<'a>(request: &'a RawRequest) -> (Option<Request<'a>>, Response) {
    let request = Request::try_from(request);
    let response = match request {
        Ok(ref request) => match (request.method(), request.path(), request.version()) {
            ("GET", "/test", "HTTP/1.1") => {
                let body = "Hello world!";

                Response::ok(Headers::new(), body)
            }
            _ => Response::not_found(Headers::new(), ""),
        },
        Err(ref e) => {
            eprintln!("{e:?}");
            let body = e.to_string();

            Response::bad_request(Headers::from([("Connection", "close")]), &body)
        }
    };

    (request.ok(), response)
}
