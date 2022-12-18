use crate::{
    message::{
        common::CommonErrorResponseData,
        ecosystem::{self, SetUserCreditResponseData},
        Message, MessageType,
    },
    model::ecosystem::EcosystemUserAccountRecord,
};
use redis::{aio::Connection, AsyncCommands, RedisError};

// 设置一个用户的余额
pub async fn set_user_credit(raw_data: serde_json::Value, redis_conn: &mut Connection) -> Message {
    let data: ecosystem::SetUserCreditRequestData = serde_json::from_value(raw_data).unwrap();

    let redis_key = format!("ecosystem:account:{}", data.user_id);
    let mut user_record = EcosystemUserAccountRecord {
        credit: 0,
        alter_records: vec![],
    };
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
    let response_data = SetUserCreditResponseData {
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
    let data: ecosystem::GetUserCreditRequestData = serde_json::from_value(raw_data).unwrap();

    let redis_key = format!("ecosystem:account:{}", data.user_id);
    let user_account_record: Option<String> = redis_conn.get(&redis_key).await.unwrap();
    if user_account_record.is_none() {
        return Message::from(CommonErrorResponseData {
            message: "user not found".to_string(),
        });
    }
    let user_account: EcosystemUserAccountRecord =
        serde_json::from_str(user_account_record.unwrap().as_str()).unwrap();
    let resp_data = ecosystem::GetUserCreditResponseData {
        user_id: data.user_id,
        credit: user_account.credit,
        alter_records: user_account.alter_records,
    };
    Message {
        message_type: MessageType::EcosytemGetUserCreditResponse,
        data: serde_json::to_value(resp_data).unwrap(),
    }
}

// 增减用户余额
pub async fn alter_user_credit(
    raw_data: serde_json::Value,
    redis_conn: &mut Connection,
) -> Message {
    let data: ecosystem::AlterUserCreditRequestData = serde_json::from_value(raw_data).unwrap();

    let redis_key = format!("ecosystem:account:{}", data.user_id);
    let user_account_record: Option<String> = redis_conn.get(&redis_key).await.unwrap();
    if user_account_record.is_none() {
        return Message::from(CommonErrorResponseData {
            message: "user not found".to_string(),
        });
    }
    let mut user_account: EcosystemUserAccountRecord =
        serde_json::from_str(user_account_record.unwrap().as_str()).unwrap();
    user_account.credit += data.credit;
    if user_account.credit < 0 {
        return Message::from(CommonErrorResponseData {
            message: "credit not enough".to_string(),
        });
    }
    let r: Result<(), RedisError> = redis_conn
        .set(&redis_key, serde_json::to_string(&user_account).unwrap())
        .await;
    if r.is_err() {
        return Message::from(r.unwrap_err());
    }
    Message {
        message_type: MessageType::EcosytemAlterUserCreditResponse,
        data: serde_json::to_value(ecosystem::AlterUserCreditResponseData {
            user_id: data.user_id,
            credit: user_account.credit,
        })
        .unwrap(),
    }
}

// 用户对用户转账
pub async fn transfer_user_credit(
    raw_data: serde_json::Value,
    redis_conn: &mut Connection,
) -> Message {
    let req: ecosystem::TransferCreditRequestData = serde_json::from_value(raw_data).unwrap();

    let from_account_redis_key = format!("ecosystem:account:{}", req.from_user_id);
    let to_account_redis_key = format!("ecosystem:account:{}", req.to_user_id);

    let from_account_record: Option<String> =
        redis_conn.get(&from_account_redis_key).await.unwrap();
    if from_account_record.is_none() {
        return Message::from(CommonErrorResponseData {
            message: "from user not found".to_string(),
        });
    }
    let to_account_record: Option<String> = redis_conn.get(&to_account_redis_key).await.unwrap();
    if to_account_record.is_none() {
        return Message::from(CommonErrorResponseData {
            message: "to user not found".to_string(),
        });
    }
    let mut from_account: EcosystemUserAccountRecord =
        serde_json::from_str(from_account_record.unwrap().as_str()).unwrap();
    if from_account.credit < req.credit {
        return Message::from(CommonErrorResponseData {
            message: "credit not enough".to_string(),
        });
    }
    let mut to_account: EcosystemUserAccountRecord =
        serde_json::from_str(to_account_record.unwrap().as_str()).unwrap();

    from_account.credit -= req.credit;
    from_account
        .alter_records
        .push(crate::model::ecosystem::EcosystemUserCreditAlterRecord {
            time: chrono::Utc::now().timestamp(),
            credit: -req.credit,
            reason: format!(
                "transfer$#$from:{}$#$to:{}",
                req.from_user_id, req.to_user_id
            ),
        });
    to_account.credit += req.credit;
    to_account
        .alter_records
        .push(crate::model::ecosystem::EcosystemUserCreditAlterRecord {
            time: chrono::Utc::now().timestamp(),
            credit: req.credit,
            reason: format!(
                "transfer$#$from:{}$#$to:{}",
                req.from_user_id, req.to_user_id
            ),
        });
    // TODO 引入transaction
    let _: () = redis_conn
        .set(
            &from_account_redis_key,
            serde_json::to_string(&from_account).unwrap(),
        )
        .await
        .unwrap();
    let _: () = redis_conn
        .set(
            &to_account_redis_key,
            serde_json::to_string(&to_account).unwrap(),
        )
        .await
        .unwrap();
    Message {
        message_type: MessageType::EcosytemTransferUserCreditResponse,
        data: serde_json::to_value(ecosystem::TransferCreditResponseData {
            from_user_id: req.from_user_id,
            from_user_credit: from_account.credit,
            to_user_id: req.to_user_id,
            to_user_credit: to_account.credit,
        })
        .unwrap(),
    }
}
