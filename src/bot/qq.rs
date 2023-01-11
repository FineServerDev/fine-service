use std::sync::Arc;

use axum::async_trait;
use futures_util::StreamExt;
use rand::{prelude::StdRng, SeedableRng};
use ricq::{
    client::{Connector, DefaultConnector},
    ext::common::after_login,
    handler::{Handler, QEvent},
    msg::MessageChain,
    Client, Device, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess,
    LoginUnknownStatus, Protocol,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::info;

pub struct FineHandler {
    super_users: Vec<u64>,
    allowed_groups: Vec<u64>,
}

impl FineHandler {
    pub fn new(super_users: Vec<u64>, allowed_groups: Vec<u64>) -> Self {
        Self {
            super_users,
            allowed_groups,
        }
    }
}

#[async_trait]
impl Handler for FineHandler {
    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::GroupMessage(m) => {
                info!(
                    "MESSAGE (GROUP={}): {}",
                    m.inner.group_code, m.inner.elements
                );
                let mut msg_chain = MessageChain::default();
                msg_chain.push(m.inner.elements.0);
                if self.super_users.contains(&(m.inner.from_uin as u64)) {
                    m.client
                        .send_group_message(m.inner.group_code, msg_chain)
                        .await
                        .ok();
                }
            }
            QEvent::FriendMessage(m) => {
                tracing::info!(
                    "MESSAGE (FRIEND={}): {}",
                    m.inner.from_uin,
                    m.inner.elements
                )
            }
            QEvent::GroupTempMessage(m) => {
                tracing::info!("MESSAGE (TEMP={}): {}", m.inner.from_uin, m.inner.elements)
            }
            QEvent::GroupRequest(m) => {
                tracing::info!(
                    "REQUEST (GROUP={}, UIN={}): {}",
                    m.inner.group_code,
                    m.inner.req_uin,
                    m.inner.message,
                );
            }
            QEvent::NewFriendRequest(m) => {
                tracing::info!("REQUEST (UIN={}): {}", m.inner.req_uin, m.inner.message)
            }
            _ => tracing::info!("{:?}", e),
        }
    }
}

pub async fn qq_bot_client(
    uin: i64,
    password: String,
    super_users: Vec<u64>,
    allowed_groups: Vec<u64>,
) {
    let mut seed = StdRng::seed_from_u64(uin as u64);
    let device = Device::random_with_rng(&mut seed);
    let f_handler = FineHandler::new(super_users, allowed_groups);
    let client = Arc::new(Client::new(device, Protocol::IPad.into(), f_handler));

    let handle = tokio::spawn({
        let client = client.clone();
        let stream = DefaultConnector.connect(&client).await.unwrap();
        async move { client.start(stream).await }
    });
    tokio::task::yield_now().await;

    let mut resp = client
        .password_login(uin, &password)
        .await
        .expect("failed to login");

    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                info!("login success: {:?}", account_info);
                break;
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref sms_phone,
                ref verify_url,
                ref message,
                ..
            }) => {
                info!("device locked: {:?}", message);
                info!("sms_phone: {:?}", sms_phone);
                info!("verify_url: {:?}", verify_url);
                info!("手机打开url, 处理完成后重启程序");
                std::process::exit(0);
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha {
                ref verify_url,
                image_captcha: ref _image_captcha,
                ..
            }) => {
                info!("滑块URL: {:?}", verify_url);
                info!("请输入ticket:");
                let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());
                let ticket = reader
                    .next()
                    .await
                    .transpose()
                    .expect("failed to read ticket")
                    .expect("failed to read ticket");
                resp = client
                    .submit_ticket(&ticket)
                    .await
                    .expect("failed to submit ticket");
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = client
                    .device_lock_login()
                    .await
                    .expect("failed to login with device lock");
            }
            LoginResponse::AccountFrozen => {
                panic!("account frozen");
            }
            LoginResponse::TooManySMSRequest => {
                panic!("too many sms request");
            }
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref tlv_map,
                ref message,
            }) => {
                panic!(
                    "unknown login status: {:?}, {:?}, {:?}",
                    message, status, tlv_map
                );
            }
        }
    }
    info!("login success, waiting for client start {:?}", resp);
    after_login(&client).await;
    handle.await.unwrap();
}
