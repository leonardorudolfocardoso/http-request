use std::{collections::HashMap, fmt::Display, io::BufRead, str::Utf8Error};

use tokio::io::{AsyncBufReadExt, AsyncReadExt};

pub type RawRequest = Vec<u8>;
pub type Headers<'a> = HashMap<&'a str, &'a str>;
pub struct Request<'a> {
    version: &'a str,
    method: &'a str,
    path: &'a str,
    headers: Headers<'a>,
}
impl<'a> Request<'a> {
    pub fn version(&self) -> &'a str {
        self.version
    }
    pub fn method(&self) -> &'a str {
        self.method
    }
    pub fn path(&self) -> &'a str {
        self.path
    }
    pub fn headers(&self) -> &Headers<'a> {
        &self.headers
    }
    pub fn should_close(&self) -> bool {
        self.headers.iter().any(|(key, value)| {
            key.eq_ignore_ascii_case("connection") && value.eq_ignore_ascii_case("close")
        })
    }
}

#[derive(Debug)]
pub enum InvalidRequest {
    Malformed,
    MalformedHeader,
    Utf8Error(Utf8Error),
}

impl Display for InvalidRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidRequest::Malformed => write!(f, "Malformed request"),
            InvalidRequest::MalformedHeader => write!(f, "Malformed header request"),
            InvalidRequest::Utf8Error(e) => write!(f, "Invalid utf8 request: {e}"),
        }
    }
}

impl<'a> TryFrom<&'a RawRequest> for Request<'a> {
    type Error = InvalidRequest;

    fn try_from(raw: &'a RawRequest) -> Result<Self, Self::Error> {
        let request_line_end = raw
            .windows(2)
            .position(|bytes| bytes == b"\r\n")
            .ok_or(InvalidRequest::Malformed)?;
        let request_line_bytes = &raw[..request_line_end];
        let request_line = str::from_utf8(request_line_bytes).map_err(InvalidRequest::Utf8Error)?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or(InvalidRequest::Malformed)?;
        let path = parts.next().ok_or(InvalidRequest::Malformed)?;
        let version = parts.next().ok_or(InvalidRequest::Malformed)?;

        let header_start = request_line_end + 2;
        let header_end = header_start
            + raw[header_start..]
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .ok_or(InvalidRequest::Malformed)?;

        let header_bytes = &raw[header_start..header_end];
        let headers_text = str::from_utf8(header_bytes).map_err(InvalidRequest::Utf8Error)?;

        let mut headers = Headers::new();

        for line in headers_text.split("\r\n") {
            let (key, value) = line
                .split_once(": ")
                .ok_or(InvalidRequest::MalformedHeader)?;
            headers.insert(key, value);
        }

        Ok(Request {
            version,
            method,
            path,
            headers,
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    IO(std::io::Error),
    Utf8(Utf8Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::IO(e) => write!(f, "ReadError: IO error: {e}"),
            ReadError::Utf8(e) => write!(f, "ReadError: Utf8 error: {e}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadStatus {
    Complete,
    Closed,
}

pub struct Reader<R: BufRead> {
    inner: R,
}

impl From<Utf8Error> for ReadError {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl<R: BufRead> Reader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
    pub fn read(&mut self, buf: &mut RawRequest) -> Result<ReadStatus, ReadError> {
        loop {
            let mut line = Vec::new();

            let n = self.inner.read_until(b'\n', &mut line)?;

            if n == 0 {
                return Ok(ReadStatus::Closed);
            }

            let end = line == b"\r\n";

            buf.extend(line);

            if end {
                break;
            }
        }

        let headers = std::str::from_utf8(buf)?;

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
        self.inner.read_exact(&mut buf[start..])?;

        Ok(ReadStatus::Complete)
    }
}

pub struct AsyncReader<R: AsyncBufReadExt + Unpin> {
    inner: R,
}

impl<R: AsyncBufReadExt + Unpin> AsyncReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
    pub async fn read(&mut self, buf: &mut RawRequest) -> Result<ReadStatus, ReadError> {
        loop {
            let mut line = Vec::new();

            let n = self.inner.read_until(b'\n', &mut line).await?;

            if n == 0 {
                return Ok(ReadStatus::Closed);
            }

            let end = line == b"\r\n";

            buf.extend(line);

            if end {
                break;
            }
        }

        let headers = std::str::from_utf8(buf)?;

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
        self.inner.read_exact(&mut buf[start..]).await?;

        Ok(ReadStatus::Complete)
    }
}

enum Status {
    NotFound,
    BadRequest,
    Ok,
}

type Code = u16;
impl Status {
    fn name(&self) -> &'static str {
        match self {
            Status::NotFound => "NOT FOUND",
            Status::BadRequest => "BAD REQUEST",
            Status::Ok => "OK",
        }
    }
    fn code(&self) -> Code {
        match self {
            Status::NotFound => 404,
            Status::BadRequest => 400,
            Status::Ok => 200,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code();
        let name = self.name();
        write!(f, "{code} {name}")
    }
}

pub struct Response(String);

impl Response {
    fn new(status: Status, headers: Headers, body: &str) -> Response {
        let mut response = format!("HTTP/1.1 {status}\r\n");

        for (key, value) in headers {
            response.push_str(&format!("{key}: {value}\r\n"));
        }

        let len = body.len();
        response.push_str(&format!("Content-Length: {len}\r\n\r\n"));

        response.push_str(body);

        Response(response)
    }

    pub fn ok(headers: Headers, body: &str) -> Response {
        Response::new(Status::Ok, headers, body)
    }
    pub fn bad_request(headers: Headers, body: &str) -> Response {
        Response::new(Status::BadRequest, headers, body)
    }
    pub fn not_found(headers: Headers, body: &str) -> Response {
        Response::new(Status::NotFound, headers, body)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parses_valid_get_request() {
        let raw = b"GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec();

        let request = Request::try_from(&raw).unwrap();

        assert_eq!(request.method(), "GET");
        assert_eq!(request.path(), "/test");
        assert_eq!(request.version(), "HTTP/1.1");
    }

    #[test]
    fn parses_headers() {
        let raw = b"GET /test HTTP/1.1\r\nHost: localhost\r\nUser-Agent: nc\r\n\r\n".to_vec();

        let request = Request::try_from(&raw).unwrap();

        assert_eq!(request.headers().get("Host"), Some(&"localhost"));
        assert_eq!(request.headers().get("User-Agent"), Some(&"nc"));
    }

    #[test]
    fn detects_connection_close_case_insensitively() {
        let raw = b"GET /test HTTP/1.1\r\ncOnNeCtIoN: Close\r\n\r\n".to_vec();

        let request = Request::try_from(&raw).unwrap();

        assert!(request.should_close());
    }

    #[test]
    fn rejects_malformed_request_line() {
        let raw = b"GET\r\nHost: localhost\r\n\r\n".to_vec();

        let result = Request::try_from(&raw);

        assert!(matches!(result, Err(InvalidRequest::Malformed)));
    }

    #[test]
    fn rejects_malformed_header() {
        let raw = b"GET /test HTTP/1.1\r\nHost localhost\r\n\r\n".to_vec();

        let result = Request::try_from(&raw);

        assert!(matches!(result, Err(InvalidRequest::MalformedHeader)));
    }

    #[test]
    fn reader_returns_closed_on_eof_before_request() {
        let mut reader = Reader::new(Cursor::new(&b""[..]));
        let mut buf = RawRequest::new();

        let status = reader.read(&mut buf).unwrap();

        assert_eq!(status, ReadStatus::Closed);
        assert!(buf.is_empty());
    }

    #[test]
    fn reader_reads_complete_header_only_request() {
        let input = b"GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut reader = Reader::new(Cursor::new(&input[..]));
        let mut buf = RawRequest::new();

        let status = reader.read(&mut buf).unwrap();

        assert_eq!(status, ReadStatus::Complete);
        assert_eq!(buf.as_slice(), input);
    }

    #[test]
    fn reader_reads_body_using_content_length() {
        let input =
            b"POST /test HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhelloextra";
        let expected = b"POST /test HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello";
        let mut reader = Reader::new(Cursor::new(&input[..]));
        let mut buf = RawRequest::new();

        let status = reader.read(&mut buf).unwrap();

        assert_eq!(status, ReadStatus::Complete);
        assert_eq!(buf.as_slice(), expected);
    }

    #[test]
    fn response_formats_ok_with_content_length() {
        let response = Response::ok(Headers::new(), "Hello world!");

        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello world!".as_bytes()
        );
    }

    #[test]
    fn response_formats_bad_request_with_custom_headers() {
        let response = Response::bad_request(Headers::from([("Connection", "close")]), "bad");

        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 400 BAD REQUEST\r\nConnection: close\r\nContent-Length: 3\r\n\r\nbad"
                .as_bytes()
        );
    }

    #[test]
    fn response_formats_not_found_with_empty_body() {
        let response = Response::not_found(Headers::new(), "");

        assert_eq!(
            response.as_bytes(),
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n".as_bytes()
        );
    }
}
