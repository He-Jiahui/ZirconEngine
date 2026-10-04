//! 包在特性目录中的身份分类；用于确定特性定义的提供包归属。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 标准包默认自有特性；特性扩展包由目录把自身 ID 视为所含特性的提供包。
pub enum PluginPackageKind {
    #[default]
    Standard,
    FeatureExtension,
}
