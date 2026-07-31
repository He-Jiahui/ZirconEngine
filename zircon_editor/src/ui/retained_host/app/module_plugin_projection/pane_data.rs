mod report;
mod view_rows;

use crate::ui::host::EditorPluginStatusReport;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::RetainedEditorHost;
use report::load_module_plugin_status_report;
use view_rows::module_plugin_status_rows;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn module_plugins_pane_data(
        &self,
        chrome: &EditorChromeSnapshot,
    ) -> ModulePluginsPaneViewData {
        let status_report = load_module_plugin_status_report(self, chrome);
        self.module_plugin_projection_cache
            .borrow_mut()
            .get_or_build(&status_report, module_plugins_pane_from_status_report)
    }
}

pub(super) fn module_plugins_pane_from_status_report(
    status_report: &EditorPluginStatusReport,
) -> ModulePluginsPaneViewData {
    ModulePluginsPaneViewData {
        plugins: model_rc(module_plugin_status_rows(status_report)),
        diagnostics: status_report.diagnostics.join("\n").into(),
    }
}
