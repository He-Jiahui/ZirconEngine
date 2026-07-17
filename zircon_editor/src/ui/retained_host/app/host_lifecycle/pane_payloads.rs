use std::collections::BTreeMap;

use super::super::*;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::workbench::snapshot::ViewContentKind;
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
        let collect_ui_asset_panes = pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::UiAssetEditor,
        );
        let collect_animation_panes = pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::AnimationSequenceEditor,
        ) || pane_payload_visibility::should_collect_payload_for_kind(
            model,
            ViewContentKind::AnimationGraphEditor,
        );
        let view_instances = if collect_ui_asset_panes || collect_animation_panes {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "collect_editor_pane_view_instances"
            );
            self.runtime.current_view_instances()
        } else {
            Vec::new()
        };
        let ui_asset_panes = if collect_ui_asset_panes {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_ui_asset_panes");
            self.collect_ui_asset_panes(&view_instances)
        } else {
            BTreeMap::new()
        };
        let animation_panes = if collect_animation_panes {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_animation_panes");
            self.collect_animation_editor_panes(&view_instances)
        } else {
            BTreeMap::new()
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn editor_pane_payloads_share_one_visible_instance_snapshot() {
        let sources = [
            include_str!("pane_payloads.rs"),
            include_str!("pane_payloads/editor_panes.rs"),
        ]
        .concat();
        let instance_snapshot = ["current_view_", "instances()"].concat();

        assert_eq!(sources.matches(&instance_snapshot).count(), 1);
        assert!(sources.contains("collect_ui_asset_panes || collect_animation_panes"));
    }
}
