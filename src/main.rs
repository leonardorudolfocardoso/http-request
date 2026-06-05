use std::{
    io::{BufReader, Write},
    net::TcpListener,
};

use http_server::{
    ReadRequest, handle, read_request,
    request::{RawRequest, Request},
};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut reader = BufReader::new(&stream);
                let mut raw = RawRequest::with_capacity(4096);
                loop {
                    raw.clear();

                    match read_request(&mut reader, &mut raw) {
                        Ok(ReadRequest::Complete) => match Request::try_from(&raw) {
                            Ok(request) => {
                                let response = handle(&request);

                                reader.get_mut().write_all(response.as_bytes()).unwrap();

                                if request.should_close() {
                                    break;
                                }
                            }
                            Err(e) => eprintln!("{e:?}"),
                        },
                        Ok(ReadRequest::Closed) => break,
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
