use serde::{Deserialize, Serialize};

use crate::core::editor_message::DocumentId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 文档生命周期和显示状态事实；打开、关闭、保存要求逐条保留，脏状态与焦点请求允许按最新态合并。
pub enum DocumentMessage {
    Opened { doc: DocumentId },
    Closed { doc: DocumentId },
    Saved { doc: DocumentId },
    DirtyChanged { doc: DocumentId, dirty: bool },
    FocusRequested { doc: DocumentId },
}
