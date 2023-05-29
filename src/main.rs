use std::net::SocketAddr;

use port_scanner::{
    local_port_available
};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
    Router, extract::{Query, Path},
};
use serde::{Deserialize, Serialize};


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/scan", get(iterative_scan))
        .route("/scan/:req_port", get(iterative_scan_path));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
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
