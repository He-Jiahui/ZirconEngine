use zircon_runtime::plugin::ProjectPluginSelection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginFeatureSelectionUpdateReport {
    pub plugin_id: String,
    pub feature_id: String,
    pub enabled: bool,
    pub project_selection: ProjectPluginSelection,
    pub enabled_dependency_plugins: Vec<String>,
    pub enabled_dependency_features: Vec<String>,
    pub diagnostics: Vec<String>,
}
