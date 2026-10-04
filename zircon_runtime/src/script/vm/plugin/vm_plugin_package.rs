//! 统一包模型同时容纳字节码与 zr_vm 项目源；发现阶段可暂留空字节码，管理器选择装载时才物化载荷。
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::vm_plugin_manifest::VmPluginManifest;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmPluginPackage {
    pub manifest: VmPluginManifest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zr_vm_project: Option<ZrVmPluginProjectSource>,
    pub bytecode: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrVmPluginProjectSource {
    pub project_path: PathBuf,
    pub entry_module: String,
    pub execution_mode: ZrVmExecutionMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrVmExecutionMode {
    Interp,
    Binary,
}

impl Default for ZrVmExecutionMode {
    fn default() -> Self {
        Self::Binary
    }
}
