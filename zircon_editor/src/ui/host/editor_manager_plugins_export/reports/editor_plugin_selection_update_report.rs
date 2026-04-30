use zircon_runtime::ProjectPluginSelection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginSelectionUpdateReport {
    pub plugin_id: String,
    pub project_selection: ProjectPluginSelection,
    pub diagnostics: Vec<String>,
}
