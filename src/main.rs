use std::{
    net::SocketAddr, sync::{Arc, Mutex},
};

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use port_scanner::local_port_available;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let msg_dump = MsgStore{
        data: Arc::new(Mutex::new("Message list".to_string()))
    };

    // build our application with a route
    let app = Router::new()
        .route("/scan", get(iterative_scan))
        .route("/", get(sub_ws))
        .route("/msg", post(save_msg))
        .route("/scan/:req_port", get(iterative_scan_path))
        .with_state(msg_dump);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct ReqPort {
    port: u16,
}

#[derive(Clone)]
struct MsgStore {
    data: Arc<Mutex<String>>
}

// basic handler that responds with a static string
async fn iterative_scan(Query(req_port): Query<ReqPort>) -> String {
    format!(
        "Port {} is {}",
        req_port.port,
        if local_port_available(req_port.port) {
            "open"
        } else {
            "closed"
        }
    )
}

// basic handler that responds with a static string
async fn iterative_scan_path(Path(req_port): Path<u16>) -> impl IntoResponse {
    let result = format!(
        "Port {} is {}",
        req_port,
        if local_port_available(req_port) {
            "open"
        } else {
            "closed"
        }
    );

    (StatusCode::OK, result)
}

async fn handle_ws(mut ws: WebSocket, addr: SocketAddr) {
    let _ = ws.send(Message::Text(format!("Hello {addr}"))).await;
}

async fn sub_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("{addr}");
    ws.on_upgrade(move |s| handle_ws(s, addr))
}

async fn save_msg(
    State(msg_history): State<MsgStore>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    data: String,
) -> impl IntoResponse {
    let new_msg = format!("{} says: {:?}", addr.ip().to_string(), data);
    println!("{new_msg}");

    let mut msg_data = msg_history.data.lock().expect("Error poisoned mutex");
    let updated_store = format!("{}\n{}", *msg_data , new_msg);
    *msg_data = updated_store.to_owned();

    (StatusCode::OK, updated_store)
}
