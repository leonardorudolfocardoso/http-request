use std::{io::BufReader, net::TcpListener};

use http_server::{serve_connection, thread::ThreadPool};

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).unwrap();
    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(move || {
                    let mut writer = stream.try_clone().unwrap();
                    let reader = BufReader::new(&stream);

                    if let Err(e) = serve_connection(reader, &mut writer) {
                        eprintln!("{e}");
                    }
                });
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
