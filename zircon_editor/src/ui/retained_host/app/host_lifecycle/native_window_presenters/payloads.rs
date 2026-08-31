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
        let (module_plugins_visible, build_export_visible) =
            pane_payload_visibility::payload_visibility_for_pair(
                model,
                ViewContentKind::ModulePlugins,
                ViewContentKind::BuildExport,
            );
        let module_plugins = if module_plugins_visible {
            self.module_plugins_pane_data(chrome)
        } else {
            ModulePluginsPaneViewData::default()
        };
        let build_export = if build_export_visible {
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
