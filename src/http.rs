use std::collections::HashMap;

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
    pub fn headers(&'a self) -> &'a Headers<'a> {
        &self.headers
    }
    pub fn should_close(&self) -> bool {
        self.headers.iter().any(|(key, value)| {
            key.eq_ignore_ascii_case("connection") && value.eq_ignore_ascii_case("close")
        })
    }
}

#[derive(Debug)]
pub struct InvalidRequest;
impl<'a> TryFrom<&'a RawRequest> for Request<'a> {
    type Error = InvalidRequest;

    fn try_from(raw: &'a RawRequest) -> Result<Self, Self::Error> {
        let request_line_end = raw
            .windows(2)
            .position(|bytes| bytes == b"\r\n")
            .ok_or(InvalidRequest)?;
        let request_line_bytes = &raw[..request_line_end];
        let request_line = str::from_utf8(request_line_bytes).unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap();
        let path = parts.next().unwrap();
        let version = parts.next().unwrap();

        let header_start = request_line_end + 2;
        let header_end = header_start
            + raw[header_start..]
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .unwrap();

        let header_bytes = &raw[header_start..header_end];
        let headers_text = str::from_utf8(header_bytes).unwrap();

        let mut headers = Headers::new();

        for line in headers_text.split("\r\n") {
            let (key, value) = line.split_once(": ").unwrap();
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
