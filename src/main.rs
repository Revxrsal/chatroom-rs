use std::net::SocketAddr;
use std::process::exit;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt};
use tokio::net::TcpSocket;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() {
    let (tx, rx) = unbounded_channel();

    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
        loop {
            let mut buf = String::new();
            reader.read_line(&mut buf).await.unwrap();
            tx.send(buf).unwrap();
        }
    });

    if let Err(why) = connect("127.0.0.1:2000", rx).await {
        eprintln!("failed to connect to server! :(");
        eprintln!("{:?}", why);
        return;
    }
}

async fn connect(address: &str, mut messages_to_send: UnboundedReceiver<String>) -> anyhow::Result<()> {
    let addr = SocketAddr::V4(address.parse().unwrap());
    let client = TcpSocket::new_v4()?;
    let stream = client.connect(addr).await?;

    let mut lines = Framed::new(stream, LinesCodec::new());
    loop {
        select! {
            // we want to send a message
            Some(msg) = messages_to_send.recv() => {
                lines.send(msg).await.expect("failed to send message");
            }
            result = lines.next() => match result {
                // A message was received from the server
                Some(Ok(msg)) => {
                    println!("{}", msg);
                }
                // The connection was terminated
                Some(Err(_)) | None => {
                    println!("server has gone offline! good bye!");
                    exit(0);
                }
            }
        }
    }
}