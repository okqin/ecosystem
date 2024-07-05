use anyhow::Result;
use axum::{response::IntoResponse, routing::get, Router};
use rand::Rng;
use std::time::Duration;
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("/tmp/", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // stdout console logger
    let console = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    // rolling file logger
    let file = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> impl IntoResponse {
    sleep(Duration::from_millis(100)).await;
    long_task().await
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    let random_number: u64 = rng.gen_range(1..=200);
    std::thread::sleep(Duration::from_millis(random_number));
    let elapsed = start.elapsed().as_millis();
    if elapsed > 100 {
        warn!(
            app.task_duration = elapsed,
            "The task is running for too long!"
        );
    }
    "Hello World!"
}
