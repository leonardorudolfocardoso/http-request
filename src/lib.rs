use std::{
    io::{BufRead, Write},
    time::Duration,
};

use crate::http::{Headers, RawRequest, ReadStatus, Reader, Request, Response};

pub mod http;
pub mod thread;

pub fn handle<'a>(request: &'a RawRequest) -> (Option<Request<'a>>, Response) {
    let request = Request::try_from(request);
    let response = match request {
        Ok(ref request) => match (request.method(), request.path(), request.version()) {
            ("GET", "/test", "HTTP/1.1") => {
                let body = "Hello world!";

                Response::ok(Headers::new(), body)
            }
            ("GET", "/sleep", "HTTP/1.1") => {
                let body = "Slept for 5s";

                std::thread::sleep(Duration::from_secs(5));
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

pub fn serve_connection<R, W>(reader: R, writer: &mut W) -> std::io::Result<()>
where
    R: BufRead,
    W: Write,
{
    let mut reader = Reader::new(reader);
    let mut request = RawRequest::with_capacity(4096);

    loop {
        request.clear();

        match reader.read(&mut request) {
            Ok(ReadStatus::Complete) => {
                let (request, response) = handle(&request);

                writer.write_all(response.as_bytes())?;

                if request.is_none_or(|req| req.should_close()) {
                    break;
                }
            }
            Ok(ReadStatus::Closed) => break,
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, ErrorKind};

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

    #[test]
    fn serve_connection_writes_bad_request_and_stops() {
        let input = Cursor::new(
            &b"GET\r\nHost: localhost\r\n\r\nGET /test HTTP/1.1\r\nHost: localhost\r\n\r\n"[..],
        );
        let mut output = Vec::new();

        serve_connection(input, &mut output).unwrap();

        assert_eq!(
            output,
            b"HTTP/1.1 400 BAD REQUEST\r\nConnection: close\r\nContent-Length: 17\r\n\r\nMalformed request"
        );
    }

    #[test]
    fn serve_connection_stops_after_connection_close_request() {
        let input = Cursor::new(&b"GET /test HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\nGET /test HTTP/1.1\r\nHost: localhost\r\n\r\n"[..]);
        let mut output = Vec::new();

        serve_connection(input, &mut output).unwrap();

        assert_eq!(
            output,
            b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello world!"
        );
    }

    #[test]
    fn serve_connection_processes_multiple_keep_alive_requests() {
        let input = Cursor::new(&b"GET /test HTTP/1.1\r\nHost: localhost\r\n\r\nGET /missing HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"[..]);
        let mut output = Vec::new();

        serve_connection(input, &mut output).unwrap();

        assert_eq!(
            output,
            b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello world!HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n"
        );
    }

    #[test]
    fn serve_connection_returns_write_errors() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(ErrorKind::BrokenPipe, "write failed"))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let input =
            Cursor::new(&b"GET /test HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"[..]);
        let mut output = FailingWriter;

        let err = serve_connection(input, &mut output).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
    }
}
