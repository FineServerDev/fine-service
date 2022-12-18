use serde::{Deserialize, Serialize};

// 经济系统中的一条记录
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemUserAccountRecord {
    pub credit: i32,
    pub alter_records: Vec<EcosystemUserCreditAlterRecord>,
}

// 用户资产变动记录
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemUserCreditAlterRecord {
    pub time: i64,
    pub credit: i32,
    pub reason: String,
}
