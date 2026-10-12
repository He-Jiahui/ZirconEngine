use super::super::super::{pane_payload_visibility, RetainedEditorHost};
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

impl RetainedEditorHost {
    pub(super) fn collect_runtime_diagnostics_payload(
        &self,
    ) -> Result<RuntimeDiagnosticsSnapshot, crate::ui::host::EditorError> {
        if self
            .runtime_diagnostics_refresh_target
            .should_collect_payload()
        {
            self.runtime_diagnostics_with_profile()
        } else {
            Ok(RuntimeDiagnosticsSnapshot::default())
        }
    }

    pub(super) fn collect_module_plugins_pane_payload(
        &self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
    ) -> Result<ModulePluginsPaneViewData, crate::ui::host::EditorError> {
        if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::ModulePlugins,
        ) {
            self.module_plugins_pane_data(chrome)
        } else {
            Ok(ModulePluginsPaneViewData::default())
        }
    }

    pub(super) fn collect_build_export_pane_payload(
        &self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
    ) -> Result<BuildExportPaneViewData, crate::ui::host::EditorError> {
        if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::BuildExport,
        ) {
            self.build_export_pane_data(chrome)
        } else {
            Ok(BuildExportPaneViewData::default())
        }
    }
}

#[cfg(test)]
#[path = "tests/panes.rs"]
mod tests;
