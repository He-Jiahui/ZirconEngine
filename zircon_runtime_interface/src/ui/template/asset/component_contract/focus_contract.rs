use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 焦点公开面分为根可聚焦、初始焦点和命名公开目标；校验器要求目标属于根或导出部件。
pub struct UiComponentFocusContract {
    #[serde(default)]
    pub root_focusable: bool,
    #[serde(default)]
    pub initial_focus: Option<String>,
    #[serde(default)]
    pub public_targets: BTreeMap<String, String>,
}
