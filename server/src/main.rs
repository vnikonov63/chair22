use std::{net::SocketAddr, sync::{Arc, Mutex}, env};

use axum::{extract::{Json, State}, http::{header::CONTENT_TYPE, Method}, routing::post, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use viva::Repl;

#[derive(Deserialize)]
struct Input {
    text: String,
}

#[derive(Serialize)]
struct Output {
    result: String,
}

struct AppState {
    repl: Mutex<Repl>
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let _ = dotenvy::dotenv();

    let state = Arc::new(AppState {
        repl: Mutex::new(Repl::new()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        .route("/eval", post(eval_handler))
        .with_state(state)
        .layer(cors);

    let default_host = if cfg!(debug_assertions) { "127.0.0.1" } else { "0.0.0.0" };
    let host = env::var("HOST").unwrap_or_else(|_| default_host.into());
    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid HOST/PORT");

    println!("Server running on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn eval_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<Input>,
) -> Json<Output> {
    let mut repl = state.repl.lock().unwrap();
    let result_str = match repl.feed(&input.text) {
        Ok(Some(s)) => s,
        Ok(None) => String::new(),
        Err(e) => format!("Error: {}", e),
    };

    Json(Output { result: result_str })
}
