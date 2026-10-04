use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;
use crate::ui::workbench::view::ViewKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 布局拖放解析接口的源身份；接受此值不代表已验证停靠许可。
pub struct DragPayload {
    pub instance_id: ViewInstanceId,
    pub kind: ViewKind,
}
