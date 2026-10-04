//! 发行形态的序列化声明，连接包清单、原生制品定位和开库前兼容性检查。
use serde::{Deserialize, Serialize};

use crate::core::framework::project::ExportPackagingStrategy;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 原生加载器据 forms、ABI 和引擎兼容范围筛选候选，再用 dist_crate 定位库。
/// 缺失的可选字段按空值反序列化，不能作为通过信任与兼容性检查的凭据。
pub struct PluginDistributionManifest {
    #[serde(default)]
    pub forms: Vec<String>,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi_version: Option<u32>,
    #[serde(default)]
    pub engine_compat: String,
    #[serde(default)]
    pub dist_crate: String,
    #[serde(default)]
    pub descriptor_symbol: String,
    #[serde(default)]
    pub runtime_entry: String,
    #[serde(default)]
    pub editor_entry: String,
    #[serde(default)]
    pub assets: Vec<String>,
}
