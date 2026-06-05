use std::{
    io::{BufReader, Write},
    net::TcpListener,
};

use http_server::{RawRequest, ReadRequest, handle, parse, read_request, should_close};

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
                        Ok(ReadRequest::Complete) => match parse(&raw) {
                            Ok(request) => {
                                let response = handle(&request);

                                reader.get_mut().write_all(response.as_bytes()).unwrap();

                                if should_close(&request) {
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
