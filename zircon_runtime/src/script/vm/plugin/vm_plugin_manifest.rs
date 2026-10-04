use super::super::CapabilitySet;
use super::management_policy::VmPluginManagementPolicy;
use serde::{Deserialize, Serialize};

/// 包身份、入口和能力的运行时快照；后端装载与宿主授权共享此值，管理策略在发现阶段先验证。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub capabilities: CapabilitySet,
    #[serde(default)]
    pub management: VmPluginManagementPolicy,
}
