use serde::{Deserialize, Serialize};

use super::ComponentPropertyDescriptor;

/// 可持久化的动态组件声明。场景导入先校验插件前缀，再由类型注册表转成运行时反射模式。
///
/// 本 DTO 拥有全部字符串和属性列表；构造与 serde 回读均不执行注册准入。
/// 持久化时保留字段文本，缺失 `properties` 时由 serde 补为空列表。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentTypeDescriptor {
    /// 完整组件类型路径，插件名前缀由注册入口核对。
    pub type_id: String,
    /// 声明的插件标识，与类型路径一并参与归属检查。
    pub plugin_id: String,
    /// 面向显示的文本，目录查找则使用类型路径。
    pub display_name: String,
    /// 按声明顺序保存的字段元数据；缺失时补为空列表。
    #[serde(default)]
    pub properties: Vec<ComponentPropertyDescriptor>,
}
