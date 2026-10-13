use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 类型注册的运行时拥有者类别；用于目录过滤与组件/资源适配器匹配。
pub enum ReflectTypeRole {
    /// 普通反射值，不承诺 ECS 组件或资源适配器。
    #[default]
    Value,
    /// 由实体拥有并经组件适配器访问。
    Component,
    /// 由 World 拥有并经资源适配器访问。
    Resource,
}
