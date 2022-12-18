use redis::RedisError;
use serde::{Deserialize, Serialize};

use self::common::CommonErrorResponseData;
// websocket事件
pub mod common;
pub mod ecosystem;

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "common_success_response")]
    CommonSuccessResponse, // 通用成功返回结构
    #[serde(rename = "common_error_response")]
    CommonErrorResponse, // 通用错误返回结构
    #[serde(rename = "eco_set_user_credit_request")]
    EcosytemSetUserCreditRequest,
    #[serde(rename = "eco_set_user_credit_response")]
    EcosytemSetUserCreditResponse,
    #[serde(rename = "eco_get_user_credit_request")]
    EcosytemGetUserCreditRequest,
    #[serde(rename = "eco_get_user_credit_response")]
    EcosytemGetUserCreditResponse,
    #[serde(rename = "eco_alter_user_credit_request")]
    EcosytemAlterUserCreditRequest,
    #[serde(rename = "eco_alter_user_credit_response")]
    EcosytemAlterUserCreditResponse,
    #[serde(rename = "eco_transfer_user_credit_request")]
    EcosytemTransferUserCreditRequest,
    #[serde(rename = "eco_transfer_user_credit_response")]
    EcosytemTransferUserCreditResponse,
    // ...
}
// 所有websockte事件的外层包裹
#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub data: serde_json::Value,
}

impl From<CommonErrorResponseData> for Message {
    fn from(data: CommonErrorResponseData) -> Self {
        Message {
            message_type: MessageType::CommonErrorResponse,
            data: serde_json::to_value(data).unwrap(),
        }
    }
}

impl From<RedisError> for Message {
    fn from(err: RedisError) -> Self {
        Message {
            message_type: MessageType::CommonErrorResponse,
            data: serde_json::to_value(CommonErrorResponseData {
                message: err.to_string(),
            })
            .unwrap(),
        }
    }
}
