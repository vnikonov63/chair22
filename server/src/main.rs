use std::{
    collections::HashMap,
    env,
    net::SocketAddr, 
    sync::{
        Arc, 
        atomic::{AtomicU64, Ordering}
    },
};

use axum::{
    extract::{Json, Path, State}, 
    http::{header::CONTENT_TYPE, Method}, 
    routing::post, 
    Router
};

use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::{Mutex, RwLock};

use viva::Repl;

#[derive(Deserialize)]
struct Input {
    text: String
}

#[derive(Serialize)]
struct Output {
    result: String
}

#[derive(Serialize, Deserialize)]
struct NewRepl {
    id: u64
}
struct AppState {
    repls: RwLock<HashMap<u64, Arc<Mutex<Repl>>>>,
    next_id: AtomicU64
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let _ = dotenvy::dotenv();

    let state = Arc::new(AppState {
        next_id: AtomicU64::new(1),
        repls: RwLock::new(HashMap::new())
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        .route("/repl", post(create_repl_handler))
        .route("/eval/:id", post(eval_handler))
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
    Path(id): Path<u64>,
    Json(input): Json<Input>,
) -> Json<Output> {
    let possible_repl = {
        let map = state.repls.read().await;
        map.get(&id).cloned()
    };

    let result_str = if let Some(repl_mutex) = possible_repl {
        let mut repl = repl_mutex.lock().await;
        match repl.feed(&input.text) {
            Ok(Some(s)) => s,
            Ok(None) => String::new(),
            Err(e) => format!("Error: {}", e),
        }
    } else {
        format!("Error: repl with id: {} is not found", id)
    };

    Json(Output { result: result_str })
}

async fn create_repl_handler (State(state): State<Arc<AppState>>) -> Json<NewRepl> {
    let id = state.next_id.fetch_add(1, Ordering::Relaxed);
    {
        let mut map = state.repls.write().await;
        map.insert(id, Arc::new(Mutex::new(Repl::new())));
    }
    // debug: print current repl ids
    {
        let map = state.repls.read().await;
        let keys: Vec<String> = map.keys().map(|k| k.to_string()).collect();
        eprintln!("current repl ids after insert: {:?}", keys);
    }

    Json(NewRepl { id })
}
