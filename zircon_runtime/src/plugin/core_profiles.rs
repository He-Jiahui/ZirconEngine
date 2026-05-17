use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCoreProfile {
    pub name: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCoreProfile {
    pub name: String,
    pub required_capabilities: Vec<String>,
}

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

impl EditorCoreProfile {
    pub fn minimal() -> Self {
        Self {
            name: "editor.core.minimal".to_string(),
            required_capabilities: [
                "editor.host.ui_shell",
                "editor.host.asset_core",
                "editor.host.scene_interaction",
                "editor.host.runtime_render_embed",
                "editor.host.plugin_management",
                "editor.host.capability_bridge",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        }
    }
}
