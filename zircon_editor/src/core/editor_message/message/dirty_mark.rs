use serde::{Deserialize, Serialize};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 随消息关联的一次视图失效请求；总线把它合并到实例的脏集合，真正刷新由宿主消费该集合完成。
pub struct EditorViewDirtyMark {
    view: ViewInstanceId,
    mask: EditorViewInvalidationMask,
}

impl EditorViewDirtyMark {
    pub fn new(view: ViewInstanceId, mask: EditorViewInvalidationMask) -> Self {
        Self { view, mask }
    }

    pub fn view(&self) -> &ViewInstanceId {
        &self.view
    }

    pub fn mask(&self) -> EditorViewInvalidationMask {
        self.mask
    }
}
