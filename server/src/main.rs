use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_stream::StreamExt;
use crate::name_gen::gen_new_name;
use futures::SinkExt;

mod name_gen;

pub struct Room {
    clients: HashMap<SocketAddr, UnboundedSender<String>>,
}

impl Room {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    pub async fn broadcast(&mut self, message: String) {
        println!("{}", message);
        for (_, tx) in self.clients.iter_mut() {
            let _ = tx.send(message.clone());
        }
    }
}

pub struct Peer {
    name: String,
    lines: Framed<TcpStream, LinesCodec>,
    rx: UnboundedReceiver<String>,
}

impl Peer {
    pub async fn new(state: Arc<Mutex<Room>>, lines: Framed<TcpStream, LinesCodec>) -> io::Result<Self> {
        // Generate a new name for the client
        let name = gen_new_name();

        // Bind the address and register it to the connected clients
        let addr = lines.get_ref().peer_addr()?;

        // - The tx will be used by Shared to broadcast messages to connected
        //   clients (i.e. send messages to them)
        // - The rx will be used by Peer to receive messages from connected
        //   clients
        let (tx, rx) = unbounded_channel();
        state.lock().await.clients.insert(addr, tx);

        // Return a new Peer
        Ok(Self { name, lines, rx })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("accepting connections on port 2000!");

    let room = Arc::new(Mutex::new(Room::new()));

    loop {
        // New client!
        let (socket, addr) = listener.accept().await.unwrap();

        // The client's room instance
        let room = Arc::clone(&room);

        tokio::spawn(async move {
            if let Err(why) = handle_connection(room, socket, addr).await {
                eprintln!("an error occurred: {:?}", why);
            }
        });
    }
}

async fn handle_connection(room: Arc<Mutex<Room>>, stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    let lines = Framed::new(stream, LinesCodec::new());

    let mut peer = Peer::new(room.clone(), lines).await?;
    peer.lines.send(format!("You are known as '{}'", peer.name())).await?;

    {
        let mut room = room.lock().await;
        room.broadcast(format!("{} has joined the chat!", peer.name())).await;
    }

    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                peer.lines.send(&msg).await?;
            }
            // This peer sent a message. Broadcast it to other peers
            result = peer.lines.next() => match result {
                Some(Ok(msg)) => {
                    if !msg.is_empty() {
                        let mut room = room.lock().await;
                        let msg = format!("<{}> {}", peer.name(), msg);
                        room.broadcast(msg).await;
                    }
                }
                // Client disconnected. Exit the loop
                Some(Err(_)) | None => break,
            }
        }
    }

    {
        let mut room = room.lock().await;
        room.clients.remove(&addr);
        room.broadcast(format!("{} has left the chat!", peer.name())).await;
    }
    Ok(())
}
