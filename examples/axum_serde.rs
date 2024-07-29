use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[derive(Debug, Clone, Serialize, Builder)]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateUser {
    age: Option<u8>,
    skills: Option<Vec<String>>,
}

#[instrument]
async fn get_user(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    let user = user.lock().unwrap().clone();
    user.into()
}

#[instrument]
async fn update_user(
    State(user): State<Arc<Mutex<User>>>,
    Json(user_update): Json<UpdateUser>,
) -> Json<User> {
    let mut user = user.lock().unwrap();
    if let Some(age) = user_update.age {
        user.age = age;
    }
    if let Some(skills) = user_update.skills {
        user.skills = skills;
    }
    user.clone().into()
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = tracing_subscriber::fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(console).init();

    let user = UserBuilder::default()
        .name("Alice".to_string())
        .age(30)
        .skills(vec!["Rust".to_string(), "Python".to_string()])
        .build()?;

    let user_state = Arc::new(Mutex::new(user));

    let addr = "0.0.0.0:8080";
    let app = Router::new()
        .route("/", get(get_user))
        .route("/", patch(update_user))
        .with_state(user_state);
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
