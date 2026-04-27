use super::editor_subsystems::OPTIONAL_EDITOR_SUBSYSTEMS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorHostMinimalContract;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorHostMinimalReport {
    loaded_capabilities: Vec<String>,
    missing_capabilities: Vec<String>,
}

const MINIMAL_CAPABILITIES: &[&str] = &[
    "editor.host.ui_shell",
    "editor.host.asset_core",
    "editor.host.scene_interaction",
    "editor.host.runtime_render_embed",
    "editor.host.plugin_management",
    "editor.host.capability_bridge",
];

pub fn editor_host_minimal_contract() -> EditorHostMinimalContract {
    EditorHostMinimalContract
}

impl EditorHostMinimalContract {
    pub fn minimal_capability_ids(&self) -> Vec<String> {
        MINIMAL_CAPABILITIES
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }

    pub fn is_minimal(&self, capability: &str) -> bool {
        MINIMAL_CAPABILITIES.contains(&capability)
    }

    pub fn is_extension_blacklisted(&self, capability: &str) -> bool {
        OPTIONAL_EDITOR_SUBSYSTEMS.contains(&capability)
    }

    pub fn self_check(&self) -> EditorHostMinimalReport {
        EditorHostMinimalReport {
            loaded_capabilities: self.minimal_capability_ids(),
            missing_capabilities: Vec::new(),
        }
    }
}

impl EditorHostMinimalReport {
    pub fn loaded_capabilities(&self) -> Vec<String> {
        self.loaded_capabilities.clone()
    }

    pub fn missing_capabilities(&self) -> &[String] {
        &self.missing_capabilities
    }
}
