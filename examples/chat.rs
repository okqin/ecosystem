use anyhow::Result;
use dashmap::DashMap;
use derive_more::Display;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const MAX_CHANNELS: usize = 500;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug, Display)]
enum Message {
    #[display("[{_0} joined the chat 😆]")]
    UserJoined(String),

    #[display("[{_0} leave the chat 🙁]")]
    UserLeft(String),

    #[display("{sender}: {content}")]
    Chat { sender: String, content: String },
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let console_layer = console_subscriber::spawn();

    let console = fmt::Layer::new()
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(console)
        .init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let state_cloned = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(state_cloned, addr, stream).await {
                error!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("What is your username").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    info!("{} connected", username);

    let mut peer = state.add(addr, username, stream).await;

    while let Some(message) = peer.stream.next().await {
        let message = match message {
            Ok(message) => message,
            Err(e) => {
                warn!("Failed to read message from {}: {}", addr, e);
                state.peers.remove(&addr);
                let message = Arc::new(Message::UserLeft(peer.username.clone()));
                state.broadcast(addr, message).await;
                info!("{} disconnected", peer.username);
                break;
            }
        };
        let message = Arc::new(Message::chat(peer.username.clone(), message));
        state.broadcast(addr, message).await;
    }

    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() != &addr {
                if let Err(e) = peer.value().send(message.clone()).await {
                    error!("Failed to send message to {}: {}", peer.key(), e);
                    self.peers.remove(peer.key());
                }
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_CHANNELS);
        self.peers.insert(addr, tx);
        let (mut stream_sender, stream_receiver) = stream.split();

        // receive messages from others and send them to the client
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        // notify other clients when a new client joins
        let msg = Arc::new(Message::UserJoined(username.clone()));
        self.broadcast(addr, msg).await;

        Peer::new(username, stream_receiver)
    }
}

impl Message {
    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl Peer {
    fn new(username: String, stream: SplitStream<Framed<TcpStream, LinesCodec>>) -> Self {
        Self { username, stream }
    }
}
