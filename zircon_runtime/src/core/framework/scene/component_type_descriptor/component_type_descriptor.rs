use serde::{Deserialize, Serialize};

use super::ComponentPropertyDescriptor;

/// 可持久化的动态组件声明。场景导入先校验插件前缀，再由类型注册表转成运行时反射模式。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentTypeDescriptor {
    pub type_id: String,
    pub plugin_id: String,
    pub display_name: String,
    #[serde(default)]
    pub properties: Vec<ComponentPropertyDescriptor>,
}
