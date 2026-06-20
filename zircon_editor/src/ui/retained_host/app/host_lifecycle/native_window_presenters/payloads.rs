use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, ViewContentKind};

use super::super::super::{pane_payload_visibility, RetainedEditorHost};

pub(super) struct NativeWindowPanePayloads {
    pub(super) module_plugins: ModulePluginsPaneViewData,
    pub(super) build_export: BuildExportPaneViewData,
    pub(super) has_component_showcase_runtime: bool,
}

impl RetainedEditorHost {
    pub(super) fn prepare_native_window_pane_payloads(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &EditorChromeSnapshot,
    ) -> NativeWindowPanePayloads {
        let module_plugins = if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::ModulePlugins,
        ) {
            self.module_plugins_pane_data(chrome)
        } else {
            ModulePluginsPaneViewData::default()
        };
        let build_export = if pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::BuildExport,
        ) {
            self.build_export_pane_data(chrome)
        } else {
            BuildExportPaneViewData::default()
        };
        let has_component_showcase_runtime =
            self.prepare_component_showcase_runtime_for_presentation(model);
        NativeWindowPanePayloads {
            module_plugins,
            build_export,
            has_component_showcase_runtime,
        }
    }
}
