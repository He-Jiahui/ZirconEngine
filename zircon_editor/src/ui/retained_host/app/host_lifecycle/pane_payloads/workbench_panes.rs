use super::super::super::{pane_payload_visibility, RetainedEditorHost};
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

impl RetainedEditorHost {
    pub(super) fn collect_runtime_diagnostics_payload(&self) -> RuntimeDiagnosticsSnapshot {
        if self
            .runtime_diagnostics_refresh_target
            .should_collect_payload()
        {
            self.runtime_diagnostics_with_profile()
        } else {
            RuntimeDiagnosticsSnapshot::default()
        }
    }

    pub(super) fn collect_module_plugins_pane_payload(
        &self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
    ) -> ModulePluginsPaneViewData {
        if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::ModulePlugins,
        ) {
            self.module_plugins_pane_data(chrome)
        } else {
            ModulePluginsPaneViewData::default()
        }
    }

    pub(super) fn collect_build_export_pane_payload(
        &self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
    ) -> BuildExportPaneViewData {
        if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::BuildExport,
        ) {
            self.build_export_pane_data(chrome)
        } else {
            BuildExportPaneViewData::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn diagnostics_payload_reuses_the_publication_time_target() {
        let source = include_str!("workbench_panes.rs");
        let function = source
            .split("fn collect_runtime_diagnostics_payload")
            .nth(1)
            .and_then(|tail| tail.split("fn collect_module_plugins_pane_payload").next())
            .expect("diagnostics payload collector");

        assert!(function.contains("self.runtime_diagnostics_refresh_target"));
        assert!(function.contains("should_collect_payload()"));
        assert!(!function.contains("RuntimeDiagnosticsRefreshTarget::None"));
        assert!(!function.contains("runtime_diagnostics_refresh_target("));
        assert!(!function.contains("tool_windows"));
    }
}
