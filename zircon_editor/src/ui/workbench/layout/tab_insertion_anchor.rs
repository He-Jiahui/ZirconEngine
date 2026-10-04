use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::TabInsertionSide;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 相邻tab身份与前后侧；插入时重新定位，目标已消失则回退末尾。
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}
