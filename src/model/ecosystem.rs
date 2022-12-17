use serde::{Deserialize, Serialize};

// 经济系统中的一条记录
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemUserRecord {
    pub credit: i32,
}
