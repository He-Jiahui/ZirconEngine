use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 抽屉登记许可分类；与原生句柄或嵌入式宿主模式分属不同维度。
pub enum WindowKind {
    Ordinary,
    DrawerCapable,
    DrawerWindow,
}
