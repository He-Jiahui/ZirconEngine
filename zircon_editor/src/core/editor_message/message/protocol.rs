use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 总线的投递范围与回调语义：Publish 按订阅，Broadcast 面向所有订阅者，Request 指定目标并执行请求处理器。
pub enum EditorMessageProtocol {
    Publish,
    Request,
    Broadcast,
}
