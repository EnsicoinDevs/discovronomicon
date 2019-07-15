#![feature(async_await)]

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0,0,0,0], 3333));
    let mut listener = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = socket.read(&mut buf).await.unwrap();

                if n == 0 {
                    return
                }
                socket.write_all(&buf[0..n]).await.unwrap();
            }
        });
    }
}
