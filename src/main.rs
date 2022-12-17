use axum::{routing::get, Router, Server};
use std::{env, net::SocketAddr, sync::Arc};

use dotenvy::dotenv;

mod handler;
mod model;
mod socket;
mod message;

pub struct FineState {
    redis_client: redis::Client,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt().pretty().init();
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());

    let redis_host = env::var("REDIS_HOST").unwrap();
    let redis_port = env::var("REDIS_PORT").unwrap();
    let redis_password = env::var("REDIS_PASSWORD").unwrap();

    let redis_client = redis::Client::open(format!(
        "redis://:{}@{}:{}/",
        redis_password, redis_host, redis_port
    ))
    .unwrap();
    let service_state = Arc::new(FineState { redis_client });

    let app = Router::new()
        .route("/socket", get(socket::socket_upgrader))
        .with_state(service_state);
    Server::bind(&format!("{}:{}", host, port).parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
