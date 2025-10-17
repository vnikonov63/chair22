use std::{net::SocketAddr, sync::{Arc, Mutex}, env};

use axum::{extract::{Json, State}, http::{header::CONTENT_TYPE, Method}, routing::post, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use boa::Repl;

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

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
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
