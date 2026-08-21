use std::collections::BTreeMap;

use serde_json::json;

use super::support::{
    animation_fixture, build_export_fixture, editor_data_fixture, module_plugins_fixture,
    pane_descriptor, runtime_diagnostics_fixture,
};
use crate::ui::layouts::windows::workbench_host_window::{
    document_pane, BuildExportPaneViewData, ModulePluginsPaneViewData, PanePayload,
};
use crate::ui::workbench::layout::{
    ActivityWindowId, DocumentNode, MainHostPageLayout, MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::{ViewHost, ViewInstance, ViewInstanceId};

#[test]
fn document_pane_projects_first_wave_pane_presentations_alongside_legacy_data() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let cases = [
        ("editor.console", "res://ui/editor/host/console_body.zui"),
        (
            "editor.inspector",
            "res://ui/editor/host/inspector_body.zui",
        ),
        (
            "editor.hierarchy",
            "res://ui/editor/host/hierarchy_body.zui",
        ),
        (
            "editor.animation_sequence",
            "res://ui/editor/host/animation_sequence_body.zui",
        ),
        (
            "editor.animation_graph",
            "res://ui/editor/host/animation_graph_body.zui",
        ),
        (
            "editor.runtime_diagnostics",
            "res://ui/editor/host/runtime_diagnostics_body.zui",
        ),
        (
            "editor.performance_timeline",
            "res://ui/editor/host/performance_timeline_body.zui",
        ),
        (
            "editor.module_plugins",
            "res://ui/editor/host/module_plugins_body.zui",
        ),
        (
            "editor.build_export_desktop",
            "res://ui/editor/host/build_export_desktop_body.zui",
        ),
        (
            "editor.generated_bottom",
            "res://ui/editor/host/generated_bottom_body.zui",
        ),
    ];

    for (descriptor_id, document_id) in cases {
        let descriptor = pane_descriptor(descriptor_id);
        let instance_id = ViewInstanceId::new(format!("{descriptor_id}#1"));
        let instance = ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: descriptor.descriptor_id.clone(),
            title: descriptor.default_title.clone(),
            serializable_payload: json!({ "path": "res://animations/hero.anim" }),
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        };
        let layout = WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![instance_id.clone()],
                    active_tab: Some(instance_id.clone()),
                }),
            }],
            ..WorkbenchLayout::default()
        };
        let chrome = EditorChromeSnapshot::build(
            editor_data_fixture(),
            &layout,
            vec![instance],
            vec![descriptor.clone()],
            None,
        );
        let model = WorkbenchViewModel::build(
            &crate::core::commands::EditorCommandRegistry::default_workbench(),
            &chrome,
        );
        let animation_panes = if descriptor_id.starts_with("editor.animation_") {
            BTreeMap::from([(instance_id.0.clone(), animation_fixture())])
        } else {
            BTreeMap::new()
        };
        let runtime_diagnostics = runtime_diagnostics_fixture();
        let module_plugins = if descriptor_id == "editor.module_plugins" {
            module_plugins_fixture()
        } else {
            ModulePluginsPaneViewData::default()
        };
        let build_export = if descriptor_id == "editor.build_export_desktop" {
            build_export_fixture()
        } else {
            BuildExportPaneViewData::default()
        };

        let pane = document_pane(
            &model,
            &chrome,
            &BTreeMap::new(),
            &animation_panes,
            Some(&runtime_diagnostics),
            &module_plugins,
            &build_export,
        );
        let pane_presentation = pane
            .pane_presentation
            .as_ref()
            .unwrap_or_else(|| panic!("expected pane presentation for `{descriptor_id}`"));

        assert_eq!(pane.id, instance_id.0.as_str());
        assert_eq!(pane.title, descriptor.default_title.as_str());
        assert_eq!(pane_presentation.body.document_id, document_id);
        assert_eq!(
            pane_presentation.body.payload_kind,
            descriptor
                .pane_template
                .as_ref()
                .expect("pane template")
                .body
                .payload_kind
        );
        if descriptor_id == "editor.runtime_diagnostics" {
            match &pane_presentation.body.payload {
                PanePayload::RuntimeDiagnosticsV1(payload) => {
                    assert_eq!(payload.summary, "3 runtime systems available");
                    assert_eq!(
                        payload.render_status,
                        "Render: wgpu-test (3 viewports, 11 frames)"
                    );
                    assert_eq!(
                        payload.ui_debug_reflector_summary,
                        "UI Debug Reflector: no active surface snapshot"
                    );
                    assert!(!payload.ui_debug_reflector_has_active_snapshot);
                }
                unexpected => panic!("expected runtime diagnostics payload, found {unexpected:?}"),
            }
        }
        if descriptor_id == "editor.performance_timeline" {
            match &pane_presentation.body.payload {
                PanePayload::PerformanceTimelineV1(payload) => {
                    assert_eq!(
                        payload.summary,
                        "Profiling active: 1 frame, 1 span, 1 counter"
                    );
                    assert_eq!(payload.frame_rows.len(), 1);
                    assert_eq!(payload.hotspot_rows.len(), 1);
                }
                unexpected => panic!("expected performance timeline payload, found {unexpected:?}"),
            }
        }
        if descriptor_id == "editor.module_plugins" {
            assert_eq!(
                pane.native_body.module_plugins.diagnostics,
                "plugin catalog ready"
            );
            match &pane_presentation.body.payload {
                PanePayload::ModulePluginsV1(payload) => {
                    assert_eq!(payload.diagnostics, "plugin catalog ready");
                    assert_eq!(payload.plugins.len(), 1);
                    assert_eq!(payload.plugins[0].plugin_id, "physics");
                    assert_eq!(
                        payload.plugins[0].primary_action_id,
                        "workbench.plugin.disable.physics"
                    );
                }
                unexpected => panic!("expected module plugins payload, found {unexpected:?}"),
            }
        }
        if descriptor_id == "editor.build_export_desktop" {
            assert_eq!(
                pane.native_body.build_export.diagnostics,
                "export catalog ready"
            );
            match &pane_presentation.body.payload {
                PanePayload::BuildExportV1(payload) => {
                    assert_eq!(payload.diagnostics, "export catalog ready");
                    assert_eq!(payload.targets.len(), 1);
                    assert_eq!(payload.targets[0].profile_name, "desktop_windows");
                    assert_eq!(payload.targets[0].status, "Ready");
                }
                unexpected => panic!("expected build export payload, found {unexpected:?}"),
            }
        }
        if descriptor_id == "editor.generated_bottom" {
            assert_eq!(
                pane.native_body.generated_bottom.status,
                "Generated editor feedback panels"
            );
            match &pane_presentation.body.payload {
                PanePayload::GeneratedBottomV1(payload) => {
                    assert_eq!(payload.status, "Generated editor feedback panels");
                }
                unexpected => panic!("expected generated bottom payload, found {unexpected:?}"),
            }
        }
    }
}
