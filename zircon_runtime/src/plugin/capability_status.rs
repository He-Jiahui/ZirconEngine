//! 插件声明的能力覆盖状态，供包注册校验、目录与运行时能力视图共享。
//! 状态行不证明已注册或已激活，使用方需结合注册报告判断。
use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;

/// Implementation state for a plugin-owned capability or optional feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Complete,
    Partial,
    Stub,
    Externalized,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 包注册报告中某项能力的实现状态声明；能力视图只从实际注册报告接收它。
/// 目标模式、参考资料和说明由包校验器检查，不代表插件已激活。
pub struct CapabilityStatusManifest {
    pub capability: String,
    pub status: CapabilityStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bevy_references: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl CapabilityStatusManifest {
    pub fn new(capability: impl Into<String>, status: CapabilityStatus) -> Self {
        Self {
            capability: capability.into(),
            status,
            target_modes: Vec::new(),
            bevy_references: Vec::new(),
            note: None,
        }
    }

    // TODO: [CR-PLUGIN-BOUNDARY-0204] 确认能力视图聚合前是否必须按此列表筛选当前目标；
    // 现有视图只保留能力与状态；下一步覆盖跨目标注册报告，明确这里是展示范围还是准入限制。
    /// 限定该状态声明覆盖的运行目标；调用方仍须同时声明包及模块所支持的目标。
    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_bevy_reference(mut self, reference: impl Into<String>) -> Self {
        self.bevy_references.push(reference.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}
