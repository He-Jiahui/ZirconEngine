use serde::{Deserialize, Serialize};

use super::NetEndpoint;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// UDP 接收结果保留发送源端点；业务层收到后仍需自行解释 payload 的协议与可信边界。
pub struct NetPacket {
    pub source: NetEndpoint,
    pub payload: Vec<u8>,
}
