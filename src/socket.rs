use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::Response,
};
use tracing::{info, log::warn};

use crate::{
    handler::ecosystem::{get_user_credit, set_user_credit},
    message::{self, common::CommonErrorResponseData, MessageType},
    FineState,
};

pub async fn socket_upgrader(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(fine_state): State<Arc<FineState>>,
) -> Response {
    info!("New WebSocket connection from {}", addr);
    ws.on_upgrade(|socket| socket_handler(socket, fine_state))
}

pub async fn socket_handler(mut socket: WebSocket, fine_state: Arc<FineState>) {
    let mut redis_conn = fine_state
        .redis_client
        .get_async_connection()
        .await
        .unwrap();
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            info!("Received message: {:?}", msg);
            if let Message::Text(msg) = msg {
                // 对消息进行初步反序列化
                let msg_recv: Result<message::Message, serde_json::Error> =
                    serde_json::from_str(&msg);
                let resp: message::Message;
                if msg_recv.is_err() {
                    resp = message::Message::from(CommonErrorResponseData {
                        message: "invalid message".to_string(),
                    });
                } else {
                    let msg = msg_recv.unwrap();
                    match msg.message_type {
                        MessageType::EcosytemSetUserCreditRequest => {
                            resp = set_user_credit(msg.data, &mut redis_conn).await;
                        }
                        MessageType::EcosytemGetUserCreditRequest => {
                            resp = get_user_credit(msg.data, &mut redis_conn).await;
                        }
                        _ => todo!(),
                    }
                }

                if socket
                    .send(Message::Text(serde_json::to_string(&resp).unwrap()))
                    .await
                    .is_err()
                {
                    warn!("Failed to send message");
                    return;
                }
            }
        } else {
            warn!("Failed to receive message: {:?}", msg.unwrap_err());
            return;
        };
    }
}
