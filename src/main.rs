use std::{
    io::{BufReader, Write},
    net::TcpListener,
};

use http_server::{
    handle,
    http::{RawRequest, ReadStatus, Reader, Request},
};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut writer = stream.try_clone().unwrap();
                let reader = BufReader::new(&stream);
                let mut raw = RawRequest::with_capacity(4096);
                let mut r = Reader::new(reader);
                loop {
                    raw.clear();

                    match r.read(&mut raw) {
                        Ok(ReadStatus::Complete) => match Request::try_from(&raw) {
                            Ok(request) => {
                                let response = handle(&request);

                                writer.write_all(response.as_bytes()).unwrap();

                                if request.should_close() {
                                    break;
                                }
                            }
                            Err(e) => eprintln!("{e:?}"),
                        },
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
