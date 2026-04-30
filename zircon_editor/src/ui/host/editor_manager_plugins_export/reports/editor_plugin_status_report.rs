use super::editor_plugin_status::EditorPluginStatus;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorPluginStatusReport {
    pub plugins: Vec<EditorPluginStatus>,
    pub diagnostics: Vec<String>,
}
