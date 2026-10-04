use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 组件只通过此表导出可供引用方调用的动作路由；内部绑定不因存在于树中而自动公开。
pub struct UiComponentBindingContract {
    #[serde(default)]
    pub public_actions: BTreeMap<String, UiPublicBindingRoute>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 公开路由可声明目标和负载种类，验证器据目标检查组件树内的公开边界。
pub struct UiPublicBindingRoute {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub payload_kind: Option<String>,
}
