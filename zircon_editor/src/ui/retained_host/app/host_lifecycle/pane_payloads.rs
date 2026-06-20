use std::collections::BTreeMap;

use super::super::*;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

mod editor_panes;
mod workbench_panes;

pub(super) struct HostLifecyclePanePayloads {
    pub(super) preset_names: Vec<String>,
    pub(super) ui_asset_panes:
        BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    pub(super) animation_panes:
        BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation>,
    pub(super) runtime_diagnostics: RuntimeDiagnosticsSnapshot,
    pub(super) module_plugins: ModulePluginsPaneViewData,
    pub(super) build_export: BuildExportPaneViewData,
}

impl RetainedEditorHost {
    pub(super) fn collect_host_lifecycle_pane_payloads(
        &self,
        model: &WorkbenchViewModel,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    ) -> HostLifecyclePanePayloads {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "recompute_collect_pane_payloads"
        );
        let preset_names = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_preset_names");
            self.runtime.preset_names()
        };
        let ui_asset_panes = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_ui_asset_panes");
            self.collect_ui_asset_panes()
        };
        let animation_panes = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_animation_panes");
            self.collect_animation_editor_panes()
        };
        let runtime_diagnostics = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "collect_runtime_diagnostics"
            );
            self.collect_runtime_diagnostics_payload(model)
        };
        let module_plugins = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "collect_module_plugins_pane"
            );
            self.collect_module_plugins_pane_payload(model, chrome)
        };
        let build_export = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_build_export_pane");
            self.collect_build_export_pane_payload(model, chrome)
        };

        HostLifecyclePanePayloads {
            preset_names,
            ui_asset_panes,
            animation_panes,
            runtime_diagnostics,
            module_plugins,
            build_export,
        }
    }
}
