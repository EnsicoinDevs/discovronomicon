#![feature(async_await)]

use tokio;
//use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::codec::Encoder;
use tokio::codec::Framed;
use tokio::net::TcpListener;
use futures::{SinkExt, StreamExt};
use bytes::BytesMut;

use std::net::SocketAddr;

mod discover_message;
use discover_message::{Message,MessageCodec};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0,0,0,0], 3333));
    let mut listener = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);
    
    let mut bytes = BytesMut::new();
    MessageCodec::new().encode(Message::Ping, &mut bytes).unwrap();
    println!("message: {:?}", bytes);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut messages = Framed::new(socket, MessageCodec::new());
            while let Some(message) = messages.next().await {
                println!("Message: {:?}", message);
            }
        });
    }
}
