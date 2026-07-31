use std::sync::Arc;

use crate::ui::host::EditorPluginStatusReport;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::super::RetainedEditorHost;

pub(super) fn load_module_plugin_status_report(
    host: &RetainedEditorHost,
    _chrome: &EditorChromeSnapshot,
) -> Arc<EditorPluginStatusReport> {
    host.editor_manager.published_plugin_status_report()
}
