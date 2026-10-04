use serde::{Deserialize, Serialize};

use super::super::ExportPackagingStrategy;
use crate::core::framework::platform::RuntimeTargetMode;

use super::default_packaging::default_packaging;
use super::default_true::default_true;
use super::ProjectPluginFeatureSelection;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 单个插件在项目中的声明；enabled 和 target_modes 决定是否参与当前目标，required 仅约束参与解析后的失败。
pub struct ProjectPluginSelection {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default = "default_packaging")]
    pub packaging: ExportPackagingStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_crate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_crate: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<ProjectPluginFeatureSelection>,
}
