//! 提供运行时与编辑器核心能力的最低静态集合，供配置和网关描述所需契约。
//! 这些集合只表达必要能力名；插件注册、提供者状态和实际激活由各自的报告判断。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 运行时基础能力要求的命名集合；配置测试用它检查运行时核心清单。
pub struct RuntimeCoreProfile {
    pub name: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 编辑器网关公开的基础能力集合；会与具体插件注册摘要分别投影。
pub struct EditorCoreProfile {
    pub name: String,
    pub required_capabilities: Vec<String>,
}

/// 建立运行时最低配置的能力名称基线；调用方须另行核对提供者和目标模式。
impl RuntimeCoreProfile {
    pub fn minimal() -> Self {
        Self {
            name: "runtime.core.minimal".to_string(),
            required_capabilities: [
                "runtime.core.lifecycle",
                "runtime.core.tasks",
                "runtime.core.time",
                "runtime.core.frame_count",
                "runtime.core.diagnostics",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        }
    }
}

/// 编辑器默认网关读取此集合；高亮输入属于运行时提供、编辑器要求的跨边界能力。
impl EditorCoreProfile {
    pub fn minimal() -> Self {
        Self {
            name: "editor.core.minimal".to_string(),
            required_capabilities: [
                "editor.host.ui_shell",
                "editor.host.asset_core",
                "editor.host.scene_interaction",
                "editor.host.runtime_render_embed",
                "runtime.editor_overlay.highlight_set",
                "editor.host.plugin_management",
                "editor.host.capability_bridge",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        }
    }
}

#[cfg(test)]
#[path = "tests/core_profiles.rs"]
mod tests;
