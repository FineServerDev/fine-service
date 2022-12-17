use serde::Serialize;

#[derive(Serialize)]
pub struct CommonSuccessResponseData {
    pub message: String,
}

#[derive(Serialize)]
pub struct CommonErrorResponseData {
    pub message: String,
}

// TODO 为各类错误实现 CommonErrorResponseData 的 From trait
impl From<String> for CommonErrorResponseData {
    fn from(message: String) -> Self {
        CommonErrorResponseData { message }
    }
}
