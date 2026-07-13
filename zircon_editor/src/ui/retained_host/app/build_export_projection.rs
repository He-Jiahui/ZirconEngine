use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::RetainedEditorHost;

mod targets;

impl RetainedEditorHost {
    pub(super) fn build_export_pane_data(
        &self,
        chrome: &EditorChromeSnapshot,
    ) -> BuildExportPaneViewData {
        let mut diagnostics = Vec::new();
        let targets = targets::build_export_targets(self, chrome, &mut diagnostics);
        let wizard_view_model = targets.first().and_then(|target| {
            self.desktop_export_wizard_sessions
                .view_model(target.preset_name.as_str())
                .cloned()
        });

        BuildExportPaneViewData {
            targets: model_rc(targets),
            diagnostics: diagnostics.join("\n").into(),
            wizard_view_model,
        }
    }
}
