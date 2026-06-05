use std::{
    io::{BufReader, Write},
    net::TcpListener,
};

use http_server::{
    handle,
    http::{RawRequest, ReadStatus, Reader},
};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut writer = stream.try_clone().unwrap();
                let mut reader = Reader::new(BufReader::new(&stream));
                let mut raw = RawRequest::with_capacity(4096);
                loop {
                    raw.clear();

                    match reader.read(&mut raw) {
                        Ok(ReadStatus::Complete) => {
                            let (request, response) = handle(&raw);

                            writer.write_all(response.as_bytes()).unwrap();

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
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
