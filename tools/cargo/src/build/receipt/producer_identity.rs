//! 签名闭包中的生产者声明。
//! 构建 owner 写入工具版本、worker 与 operation；核验者可将工件追溯到具体生产操作。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerIdentity {
    pub tool: String,
    pub tool_version: String,
    pub worker_id: String,
    pub operation_id: String,
}
