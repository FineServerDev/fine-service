use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EcosystemSetUserCreditRequestData {
    pub user_id: String,
    pub credit: i32,
}

#[derive(Serialize)]
pub struct EcosystemSetUserCreditResponseData {
    pub user_id: String,
    pub credit: i32, // 返回修改后的值
}

#[derive(Deserialize)]
pub struct EcosystemGetUserCreditRequestData {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct EcosystemGetUserCreditResponseData {
    pub user_id: String,
    pub credit: i32,
}
