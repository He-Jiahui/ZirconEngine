use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Placeholder 才会尝试指定回退资源；None 与 Optional 不提供替代句柄，缺失容忍由解析阶段体现。
/// 模式是声明意图；Runtime 仅在主资源失败且模式为 Placeholder 时尝试查找回退 URI。
pub enum UiResourceFallbackMode {
    #[default]
    None,
    Placeholder,
    Optional,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 占位模式的 URI 在静态校验时要求有效且资源种类匹配，解析器随后才查找对应句柄。
pub struct UiResourceFallbackPolicy {
    #[serde(default)]
    /// 缺失时采用默认的 None 模式；默认值不意味着声明已验证。
    pub mode: UiResourceFallbackMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 拥有的可选回退 URI；None 被省略，Some 的内容仍须显式验证。
    pub uri: Option<String>,
}
