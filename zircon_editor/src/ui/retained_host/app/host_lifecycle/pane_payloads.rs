use std::collections::BTreeMap;

use super::super::*;
use crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot;
use crate::ui::layouts::windows::workbench_host_window::{
    find_tab_snapshot, BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::ViewInstanceId;
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
    pub(super) template_v2_data: BTreeMap<String, EditorUiTemplatePaneDataSnapshot>,
}

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

    #[test]
    fn shell_content_payloads_are_gated_by_the_target_kind() {
        let source = include_str!("pane_payloads.rs");
        let targeted = source
            .split("fn collect_shell_content_pane_payloads")
            .nth(1)
            .and_then(|body| body.split("fn collect_host_lifecycle_pane_payloads").next())
            .expect("targeted shell content payload collector");

        for kind in [
            "ViewContentKind::UiAssetEditor",
            "ViewContentKind::AnimationSequenceEditor",
            "ViewContentKind::AnimationGraphEditor",
            "ViewContentKind::RuntimeDiagnostics",
            "ViewContentKind::PerformanceTimeline",
            "ViewContentKind::ModulePlugins",
            "ViewContentKind::BuildExport",
        ] {
            assert!(targeted.contains(kind), "missing targeted gate for {kind}");
        }
        assert!(!targeted.contains("should_collect_payload_for_kind"));
        assert!(!targeted.contains("current_view_instances"));
        assert!(!targeted.contains("ui_template_pane_data_snapshots()"));
        assert!(targeted.contains("target_instance_id"));
    }
}
