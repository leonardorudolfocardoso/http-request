use std::{collections::HashMap, fmt::Display, io::BufRead, str::Utf8Error};

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
