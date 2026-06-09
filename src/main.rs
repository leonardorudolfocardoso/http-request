use http_server::async_serve_connection;
use tokio::{io::BufReader, net::TcpListener};

fn main() {
    let addr = "127.0.0.1:8080";
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    rt.block_on(async {
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    rt.spawn(async {
                        let (reader, mut writer) = stream.into_split();
                        let reader = BufReader::new(reader);

                        if let Err(e) = async_serve_connection(reader, &mut writer).await {
                            eprintln!("{e}");
                        }
                    });
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    });
}
