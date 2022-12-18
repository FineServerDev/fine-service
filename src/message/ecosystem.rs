use serde::{Deserialize, Serialize};

use crate::model::ecosystem::EcosystemUserCreditAlterRecord;

// 修改用户账户余额的报文载荷
#[derive(Deserialize)]
pub struct SetUserCreditRequestData {
    pub user_id: String,
    pub credit: i32,
    #[serde(default = "default_resource")]
    pub reason: String,
}

// 修改用户余额的返回报文载荷
#[derive(Serialize)]
pub struct SetUserCreditResponseData {
    pub user_id: String,
    pub credit: i32, // 返回修改后的值
}

fn default_resource() -> String {
    "".to_string()
}

// 获取用户余额的报文载荷
#[derive(Deserialize)]
pub struct GetUserCreditRequestData {
    pub user_id: String,
}

// 获取用户余额的返回报文载荷
#[derive(Serialize)]
pub struct GetUserCreditResponseData {
    pub user_id: String,
    pub credit: i32,
    pub alter_records: Vec<EcosystemUserCreditAlterRecord>,
}

// 增加或减少用户余额的报文载荷
#[derive(Deserialize)]
pub struct AlterUserCreditRequestData {
    pub user_id: String,
    pub credit: i32,
    #[serde(default = "default_resource")]
    pub reason: String,
}

// 增加或减少用户余额的返回报文载荷
#[derive(Serialize)]
pub struct AlterUserCreditResponseData {
    pub user_id: String,
    pub credit: i32,
}

// 用户转账请求报文载荷
#[derive(Deserialize)]
pub struct TransferCreditRequestData {
    pub from_user_id: String,
    pub to_user_id: String,
    pub credit: i32,
}

// 用户转账返回报文载荷
#[derive(Serialize)]
pub struct TransferCreditResponseData {
    pub from_user_id: String,
    pub from_user_credit: i32, // 转账后的余额
    pub to_user_id: String,
    pub to_user_credit: i32,
}
