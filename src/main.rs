use axum::{routing::get, Router, Server};

use std::{env, net::SocketAddr, sync::Arc};

use dotenvy::dotenv;

mod bot;
mod handler;
mod message;
mod model;
mod socket;

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

    // qq client
    let uin: i64 = env::var("UIN")
        .expect("failed to read uin")
        .parse()
        .expect("illegal uin");
    let password = env::var("PASSWORD").expect("failed to read password");
    let super_users = env::var("SUPER_USERS")
        .expect("failed to read super users")
        .split(',')
        .map(|s| s.parse::<u64>().unwrap())
        .collect::<Vec<_>>();
    let allowed_groups = env::var("ALLOWED_GROUPS")
        .expect("failed to read allowed groups")
        .split(',')
        .map(|s| s.parse::<u64>().unwrap())
        .collect::<Vec<_>>();

    tokio::spawn(bot::qq::qq_bot_client(
        uin,
        password,
        super_users,
        allowed_groups,
    ));

    // redis client
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
