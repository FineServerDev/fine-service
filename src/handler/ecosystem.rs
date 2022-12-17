use crate::{
    message::{
        common::CommonErrorResponseData,
        ecosystem::{self, EcosystemSetUserCreditResponseData},
        Message, MessageType,
    },
    model::ecosystem::EcosystemUserRecord,
};
use redis::{aio::Connection, AsyncCommands, RedisError};

// 设置一个用户的余额
pub async fn set_user_credit(raw_data: serde_json::Value, redis_conn: &mut Connection) -> Message {
    let data: ecosystem::EcosystemSetUserCreditRequestData =
        serde_json::from_value(raw_data).unwrap();

    let redis_key = format!("ecosystem:account:{}", data.user_id);
    let mut user_record = EcosystemUserRecord { credit: 0 };
    let user_record_json: String = redis_conn.get(&redis_key).await.unwrap_or_default();
    if user_record_json != "" {
        user_record = serde_json::from_str(&user_record_json).unwrap();
    }
    user_record.credit = data.credit;
    // TODO 完善错误处理
    let r: Result<(), RedisError> = redis_conn
        .set(&redis_key, serde_json::to_string(&user_record).unwrap())
        .await;
    if r.is_err() {
        return Message::from(r.unwrap_err());
    }
    let response_data = EcosystemSetUserCreditResponseData {
        user_id: data.user_id,
        credit: user_record.credit,
    };
    Message {
        message_type: MessageType::EcosytemSetUserCreditResponse,
        data: serde_json::to_value(response_data).unwrap(),
    }
}

// 获取一个用户的余额
pub async fn get_user_credit(raw_data: serde_json::Value, redis_conn: &mut Connection) -> Message {
    let data: ecosystem::EcosystemGetUserCreditRequestData =
        serde_json::from_value(raw_data).unwrap();

    let redis_key = format!("ecosystem:account:{}", data.user_id);
    let user_account_record: Option<String> = redis_conn.get(&redis_key).await.unwrap();
    if user_account_record.is_none() {
        return Message::from(CommonErrorResponseData {
            message: "user not found".to_string(),
        });
    }
    let user_account: EcosystemUserRecord =
        serde_json::from_str(user_account_record.unwrap().as_str()).unwrap();
    let resp_data = ecosystem::EcosystemGetUserCreditResponseData {
        user_id: data.user_id,
        credit: user_account.credit,
    };
    Message {
        message_type: MessageType::EcosytemGetUserCreditResponse,
        data: serde_json::to_value(resp_data).unwrap(),
    }
}
