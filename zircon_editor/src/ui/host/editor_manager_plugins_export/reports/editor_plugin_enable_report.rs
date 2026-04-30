use zircon_runtime::ProjectPluginSelection;

use super::super::super::editor_capabilities::EditorCapabilitySnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginEnableReport {
    pub plugin_id: String,
    pub enabled: bool,
    pub project_selection: ProjectPluginSelection,
    pub editor_capabilities: Vec<String>,
    pub capability_snapshot: EditorCapabilitySnapshot,
    pub diagnostics: Vec<String>,
}
