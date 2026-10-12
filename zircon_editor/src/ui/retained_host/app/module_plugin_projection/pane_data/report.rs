use std::sync::Arc;

use crate::ui::host::EditorPluginStatusReport;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::super::RetainedEditorHost;

pub(super) fn load_module_plugin_status_report(
    host: &RetainedEditorHost,
    _chrome: &EditorChromeSnapshot,
) -> Result<Arc<EditorPluginStatusReport>, crate::ui::host::EditorError> {
    host.editor_manager.enter().map_err(crate::ui::host::EditorError::from).map(|manager| manager.published_plugin_status_report())
}
