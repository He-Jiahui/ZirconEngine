use std::collections::BTreeMap;

use super::super::*;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

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
            if runtime_diagnostics_visibility::should_collect_runtime_diagnostics(model) {
                self.runtime_diagnostics_with_profile()
            } else {
                RuntimeDiagnosticsSnapshot::default()
            }
        };
        let module_plugins = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "collect_module_plugins_pane"
            );
            if pane_payload_visibility::should_collect_payload_for_kind(
                model,
                ViewContentKind::ModulePlugins,
            ) {
                self.module_plugins_pane_data(chrome)
            } else {
                ModulePluginsPaneViewData::default()
            }
        };
        let build_export = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_build_export_pane");
            if pane_payload_visibility::should_collect_payload_for_kind(
                model,
                ViewContentKind::BuildExport,
            ) {
                self.build_export_pane_data(chrome)
            } else {
                BuildExportPaneViewData::default()
            }
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

    fn collect_ui_asset_panes(
        &self,
    ) -> BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation> {
        self.runtime
            .current_view_instances()
            .into_iter()
            .filter(|instance| instance.descriptor_id.0 == "editor.ui_asset")
            .filter_map(|instance| {
                self.editor_manager
                    .ui_asset_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0, presentation))
            })
            .collect()
    }

    fn collect_animation_editor_panes(
        &self,
    ) -> BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation> {
        self.runtime
            .current_view_instances()
            .into_iter()
            .filter(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.animation_sequence" | "editor.animation_graph"
                )
            })
            .filter_map(|instance| {
                self.editor_manager
                    .animation_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0, presentation))
            })
            .collect()
    }
}
