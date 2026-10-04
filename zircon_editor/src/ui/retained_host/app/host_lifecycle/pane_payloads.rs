use std::collections::BTreeMap;

use super::super::*;
use crate::ui::layouts::windows::workbench_host_window::{
    find_tab_snapshot, BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::retained_host::app::committed_shell_state::HostLifecyclePanePayloads;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

mod editor_panes;
mod workbench_panes;

impl RetainedEditorHost {
    pub(super) fn collect_shell_content_pane_payloads(
        &self,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
        target_kind: ViewContentKind,
        target_instance_id: Option<&str>,
    ) -> HostLifecyclePanePayloads {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "recompute_collect_shell_content_payloads"
        );
        let preset_names = self.runtime.preset_names();
        let collect_ui_asset_panes = target_kind == ViewContentKind::UiAssetEditor;
        let collect_animation_panes = matches!(
            target_kind,
            ViewContentKind::AnimationSequenceEditor | ViewContentKind::AnimationGraphEditor
        );
        let ui_asset_panes = if collect_ui_asset_panes {
            target_instance_id
                .and_then(|instance_id| {
                    let view_id = ViewInstanceId::new(instance_id);
                    self.editor_manager
                        .ui_asset_editor_pane_presentation(&view_id)
                        .ok()
                        .map(|presentation| (instance_id.to_owned(), presentation))
                })
                .into_iter()
                .collect()
        } else {
            BTreeMap::new()
        };
        let animation_panes = if collect_animation_panes {
            target_instance_id
                .and_then(|instance_id| {
                    let view_id = ViewInstanceId::new(instance_id);
                    self.editor_manager
                        .animation_editor_pane_presentation(&view_id)
                        .ok()
                        .map(|presentation| (instance_id.to_owned(), presentation))
                })
                .into_iter()
                .collect()
        } else {
            BTreeMap::new()
        };
        let runtime_diagnostics = if matches!(
            target_kind,
            ViewContentKind::RuntimeDiagnostics | ViewContentKind::PerformanceTimeline
        ) {
            self.runtime_diagnostics_with_profile()
        } else {
            RuntimeDiagnosticsSnapshot::default()
        };
        let module_plugins = if target_kind == ViewContentKind::ModulePlugins {
            self.module_plugins_pane_data(chrome)
        } else {
            ModulePluginsPaneViewData::default()
        };
        let build_export = if target_kind == ViewContentKind::BuildExport {
            self.build_export_pane_data(chrome)
        } else {
            BuildExportPaneViewData::default()
        };
        let template_v2_data = target_instance_id
            .and_then(|instance_id| find_tab_snapshot(chrome, instance_id))
            .and_then(|tab| tab.pane_template.as_ref())
            .and_then(|template| {
                self.runtime
                    .ui_template_pane_data_snapshot(&template.body.document_id)
                    .map(|snapshot| (template.body.document_id.clone(), snapshot))
            })
            .into_iter()
            .collect();

        HostLifecyclePanePayloads {
            preset_names,
            ui_asset_panes,
            animation_panes,
            runtime_diagnostics,
            module_plugins,
            build_export,
            template_v2_data,
        }
    }

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
        let (ui_asset_instance_ids, animation_instance_ids) =
            if collect_ui_asset_panes || collect_animation_panes {
                zircon_runtime::profile_scope!(
                    "editor",
                    "retained_host",
                    "collect_editor_pane_instance_ids"
                );
                self.runtime
                    .editor_pane_instance_ids(collect_ui_asset_panes, collect_animation_panes)
            } else {
                (Vec::new(), Vec::new())
            };
        let (ui_asset_panes, animation_panes) = if collect_ui_asset_panes || collect_animation_panes
        {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_editor_panes");
            self.collect_editor_panes(ui_asset_instance_ids, animation_instance_ids)
        } else {
            (BTreeMap::new(), BTreeMap::new())
        };
        let runtime_diagnostics = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "collect_runtime_diagnostics"
            );
            self.collect_runtime_diagnostics_payload()
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
        let template_v2_data = {
            zircon_runtime::profile_scope!("editor", "retained_host", "collect_template_v2_data");
            self.runtime.ui_template_pane_data_snapshots()
        };

        HostLifecyclePanePayloads {
            preset_names,
            ui_asset_panes,
            animation_panes,
            runtime_diagnostics,
            module_plugins,
            build_export,
            template_v2_data,
        }
    }
}

#[cfg(test)]
#[path = "tests/pane_payloads_performance_tests.rs"]
mod performance_tests;
