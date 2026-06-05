use std::{io::BufReader, net::TcpListener};

use http_server::serve_connection;

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut writer = stream.try_clone().unwrap();
                let reader = BufReader::new(&stream);

                serve_connection(reader, &mut writer);
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
