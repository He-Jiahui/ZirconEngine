use serde::{Deserialize, Serialize};

use super::EditorMessage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 请求处理器返回的消息结果；handled 只封装结果，目标复核与结果携带的脏标记由总线完成。
pub struct EditorMessageResponse {
    message: EditorMessage,
}

impl EditorMessageResponse {
    pub fn handled(message: EditorMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &EditorMessage {
        &self.message
    }
}
