use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 相对相邻实例的tab顺序，供身份锚点重新定位后解释。
pub enum TabInsertionSide {
    Before,
    After,
}
