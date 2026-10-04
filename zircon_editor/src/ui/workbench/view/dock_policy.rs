//! 视图声明的停靠许可；实际拖放及附着由布局管理与宿主检查后执行。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockPolicy {
    DrawerOnly,
    DocumentOnly,
    DrawerOrDocument,
}
