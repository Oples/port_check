use std::net::SocketAddr;

use port_scanner::{
    local_port_available
};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
    Router, extract::{Query, Path, connect_info::ConnectInfo},
};
use serde::{Deserialize};


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/scan", get(iterative_scan))
        .route("/", get(sub_ws))
        .route("/scan/:req_port", get(iterative_scan_path));

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

// basic handler that responds with a static string
async fn iterative_scan(Query(req_port): Query<ReqPort>) -> String {
    format!("Port {} is {}", req_port.port, if local_port_available(req_port.port) { "open" } else  {"closed"})
}

// basic handler that responds with a static string
async fn iterative_scan_path(Path(req_port): Path<u16>) -> impl IntoResponse {
    let result = format!("Port {} is {}", req_port,  if local_port_available(req_port) { "open" } else  {"closed"});

    (StatusCode::OK, result)
}


async fn handle_ws(mut ws: WebSocket, addr: SocketAddr) {
    let _ = ws.send(Message::Text(format!("Hello {addr}"))).await;
}

async fn sub_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> impl IntoResponse {
    println!("{addr}");
    ws.on_upgrade(move |s| handle_ws(s, addr))
}
