use anyhow::Result;
use axum::{extract::Request, response::IntoResponse, routing::get, Router};
use once_cell::sync::Lazy;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{Config, Tracer},
    Resource,
};
use std::time::Duration;
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "axum-tracing-otlp-example",
    )])
});

#[tokio::main]
async fn main() -> Result<()> {
    // console layer for tracing-subscriber
    let console = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    // file appender layer for tracing-subscriber
    let file_appender = tracing_appender::rolling::daily("/tmp/", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::WARN);

    // opentelemetry tracing layer for tracing-subscriber
    let tracer = init_tracer()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(telemetry)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(name = "index_api", skip(req), fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> impl IntoResponse {
    sleep(Duration::from_millis(100)).await;
    multi_task().await
}

#[instrument]
async fn multi_task() -> &'static str {
    let start = Instant::now();
    let sl = sleep(Duration::from_millis(50));
    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    tokio::join!(sl, t1, t2, t3);
    let elapsed = start.elapsed().as_millis();
    info!("All tasks are done in {} ms", elapsed);
    if elapsed > 200 {
        warn!(app.duration = elapsed, "The response time is too long!");
    }
    "Hello World!"
}

#[instrument]
async fn task1() {
    sleep(Duration::from_millis(50)).await;
    info!("Task 1 is done!");
}

#[instrument]
async fn task2() {
    sleep(Duration::from_millis(100)).await;
    info!("Task 2 is done!");
}

#[instrument]
async fn task3() {
    sleep(Duration::from_millis(150)).await;
    info!("Task 3 is done!");
}

fn init_tracer() -> Result<Tracer> {
    let grpc_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://127.0.0.1:4317");

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(grpc_exporter)
        .with_trace_config(Config::default().with_resource(RESOURCE.clone()))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    Ok(tracer)
}
