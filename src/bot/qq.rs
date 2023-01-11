use std::sync::Arc;

use futures_util::StreamExt;
use rand::{prelude::StdRng, SeedableRng};
use ricq::{
    client::{Connector, DefaultConnector},
    ext::common::after_login,
    handler::DefaultHandler,
    Client, Device, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess,
    LoginUnknownStatus, Protocol,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::info;

pub async fn qq_bot(uin: i64, password: String) {
    let mut seed = StdRng::seed_from_u64(uin as u64);
    let device = Device::random_with_rng(&mut seed);
    let client = Arc::new(Client::new(device, Protocol::IPad.into(), DefaultHandler));

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
                info!("手机打开url，处理完成后重启程序");
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
