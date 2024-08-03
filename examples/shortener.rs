use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

const MAX_CONN: u32 = 100;

const LISTEN_ADDR: &str = "127.0.0.1:1234";

#[derive(Debug, Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenResponse {
    url: String,
}

#[derive(Debug, FromRow)]
struct UrlRow {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let logs_layer = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(logs_layer).init();

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Server listening on: {}", LISTEN_ADDR);

    let db_url = "postgres://localhost:5432/shortener";
    let app_state = AppState::try_new(db_url).await?;

    info!("Connected to database: {}", db_url);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(app_state);

    axum::serve(listener, app).await?;

    Ok(())
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let db_pool = PgPoolOptions::new()
            .max_connections(MAX_CONN)
            .connect(url)
            .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS urls (id CHAR(6) PRIMARY KEY, url TEXT NOT NULL UNIQUE)",
        )
        .execute(&db_pool)
        .await?;
        Ok(Self { db: db_pool })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid!(6);
        let row: UrlRow = sqlx::query_as("INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id")
            .bind(&id)
            .bind(url)
            .fetch_one(&self.db)
            .await?;
        Ok(row.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let row: UrlRow = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(row.url)
    }
}

#[instrument]
async fn shorten(
    State(state): State<AppState>,
    Json(url): Json<ShortenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .shorten(&url.url)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let body = Json(ShortenResponse {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

#[instrument]
async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Redirect::permanent(&url))
}
