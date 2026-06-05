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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_returns_ok_for_get_test() {
        let raw = b"GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec();

        let (request, response) = handle(&raw);

        assert!(request.is_some());
        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello world!".as_bytes()
        );
    }

    #[test]
    fn handle_returns_not_found_for_unknown_route() {
        let raw = b"GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec();

        let (request, response) = handle(&raw);

        assert!(request.is_some());
        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n".as_bytes()
        );
    }

    #[test]
    fn handle_returns_bad_request_for_malformed_request() {
        let raw = b"GET\r\nHost: localhost\r\n\r\n".to_vec();

        let (request, response) = handle(&raw);

        assert!(request.is_none());
        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 400 BAD REQUEST\r\nConnection: close\r\nContent-Length: 17\r\n\r\nMalformed request"
                .as_bytes()
        );
    }
}
