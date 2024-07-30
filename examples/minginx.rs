use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

fn resolve_config() -> Result<Config> {
    let config = Config {
        listen_addr: "0.0.0.0:9090".to_string(),
        upstream_addr: "127.0.0.1:8080".to_string(),
    };
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(console).init();

    let config = resolve_config()?;
    let config = Arc::new(config);
    info!("linten_addr: {}", config.listen_addr);
    info!("upstream_addr: {}", config.upstream_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let config = Arc::clone(&config);
        tokio::spawn(async move {
            match TcpStream::connect(&config.upstream_addr).await {
                Ok(upstream) => {
                    if let Err(e) = proxy(stream, upstream).await {
                        error!("Error proxy data: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Error connect to upstream: {:?}", e);
                }
            }
            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();

    let client_to_upstream = tokio::io::copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = tokio::io::copy(&mut upstream_reader, &mut client_writer);

    tokio::try_join!(client_to_upstream, upstream_to_client)?;
    Ok(())
}
