use std::fs;

use super::support::pane_body_path;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::template_runtime::EditorUiHostRuntime;

#[test]
fn performance_timeline_body_exposes_capture_export_and_summary_sections() {
    let source = fs::read_to_string(pane_body_path("performance_timeline_body.zui"))
        .expect("performance timeline pane body asset should be readable");
    let document = zircon_runtime::ui::v2::UiV2AssetLoader::load_toml_str(&source)
        .expect("performance timeline pane body asset should parse");
    let root_node_id = document
        .root
        .as_ref()
        .map(|root| root.node.as_str())
        .expect("performance timeline body should declare a root node");
    let root_node = document
        .nodes
        .get(root_node_id)
        .expect("performance timeline root node should exist");

    for control_id in [
        "PerformanceTimelineSummary",
        "PerformanceTimelineCaptureControls",
        "PerformanceTimelineFrameList",
        "PerformanceTimelineSpanSummary",
        "PerformanceTimelineHotspots",
    ] {
        assert!(
            document
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "performance timeline body should expose `{control_id}`"
        );
    }
    assert!(
        root_node.children.iter().any(|child| {
            child.slot.get("name").and_then(toml::Value::as_str)
                == Some("performance_timeline_frame_list")
        }),
        "performance timeline body should declare the frame-list native slot"
    );
}

#[test]
fn runtime_diagnostics_body_exposes_compact_runtime_and_reflector_sections() {
    let source = fs::read_to_string(pane_body_path("runtime_diagnostics_body.zui"))
        .expect("runtime diagnostics pane body asset should be readable");
    let document = zircon_runtime::ui::v2::UiV2AssetLoader::load_toml_str(&source)
        .expect("runtime diagnostics pane body asset should parse");

    for control_id in [
        "RuntimeDiagnosticsSummary",
        "FocusDiagnostics",
        "UiDebugReflectorNodeList",
    ] {
        assert!(
            document
                .nodes
                .values()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "runtime diagnostics body should expose `{control_id}`"
        );
    }
}

#[test]
fn builtin_hybrid_pane_body_documents_declare_stable_native_slot_names() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let cases = [
        (
            "hierarchy_body.zui",
            "editor.host.res://ui/editor/host/hierarchy_body.zui",
            "hierarchy_tree_slot",
        ),
        (
            "animation_sequence_body.zui",
            "editor.host.pane.animation_sequence.body",
            "animation_timeline_slot",
        ),
        (
            "animation_graph_body.zui",
            "editor.host.pane.animation_graph.body",
            "animation_graph_canvas_slot",
        ),
        (
            "module_plugins_body.zui",
            "editor.host.res://ui/editor/host/module_plugins_body.zui",
            "module_plugin_list_slot",
        ),
        (
            "build_export_desktop_body.zui",
            "editor.host.pane.build_export.body",
            "build_export_targets_slot",
        ),
    ];

    for (file_name, component_id, slot_name) in cases {
        let source = fs::read_to_string(pane_body_path(file_name))
            .unwrap_or_else(|error| panic!("failed to read `{file_name}`: {error}"));
        let document = zircon_runtime::ui::v2::UiV2AssetLoader::load_toml_str(&source)
            .unwrap_or_else(|error| panic!("failed to parse `{file_name}`: {error}"));
        let root_node_id = document
            .root
            .as_ref()
            .map(|root| root.node.as_str())
            .unwrap_or_else(|| panic!("missing root node in `{file_name}`"));
        let root_node = document
            .nodes
            .get(root_node_id)
            .unwrap_or_else(|| panic!("missing root node `{root_node_id}` in `{file_name}`"));

        assert!(
            source.contains(&format!("slot_name = \"{slot_name}\"")),
            "component `{component_id}` in `{file_name}` must declare slot `{slot_name}`"
        );
        assert!(
            root_node.children.iter().any(|child| {
                child.slot.get("name").and_then(toml::Value::as_str) == Some(slot_name)
            }),
            "component `{component_id}` in `{file_name}` must expose slot placeholder `{slot_name}` in its root children"
        );
    }
}

#[test]
fn builtin_pane_body_bindings_stay_in_expected_command_namespaces() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let cases = [
        (
            "res://ui/editor/host/console_body.zui",
            "ConsolePaneBody/ClearConsole",
            "EditorOperation",
        ),
        (
            "res://ui/editor/host/inspector_body.zui",
            "InspectorPaneBody/ApplyDraft",
            "DraftCommand",
        ),
        (
            "res://ui/editor/host/hierarchy_body.zui",
            "HierarchyPaneBody/SelectRoot",
            "SelectionCommand",
        ),
        (
            "res://ui/editor/host/animation_sequence_body.zui",
            "AnimationSequencePaneBody/ScrubTimeline",
            "AnimationCommand",
        ),
        (
            "res://ui/editor/host/animation_graph_body.zui",
            "AnimationGraphPaneBody/AddNode",
            "AnimationCommand",
        ),
        (
            "res://ui/editor/host/runtime_diagnostics_body.zui",
            "RuntimeDiagnosticsPaneBody/FocusDiagnostics",
            "DockCommand",
        ),
        (
            "res://ui/editor/host/performance_timeline_body.zui",
            "PerformanceTimelinePaneBody/RefreshSnapshot",
            "DockCommand",
        ),
        (
            "res://ui/editor/host/module_plugins_body.zui",
            "ModulePluginsPaneBody/FocusModulePlugins",
            "DockCommand",
        ),
        (
            "res://ui/editor/host/build_export_desktop_body.zui",
            "BuildExportPaneBody/FocusBuildExport",
            "DockCommand",
        ),
    ];

    for (document_id, binding_id, expected_namespace) in cases {
        let projection = runtime.project_document(document_id).unwrap();
        let binding = projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == binding_id)
            .unwrap_or_else(|| panic!("missing binding `{binding_id}` in `{document_id}`"));

        let actual_namespace = match &binding.binding.payload {
            EditorUiBindingPayload::EditorOperation { .. } => "EditorOperation",
            EditorUiBindingPayload::DockCommand(_) => "DockCommand",
            EditorUiBindingPayload::DraftCommand(_) => "DraftCommand",
            EditorUiBindingPayload::SelectionCommand(_) => "SelectionCommand",
            EditorUiBindingPayload::AnimationCommand(_) => "AnimationCommand",
            other => panic!(
                "binding `{binding_id}` in `{document_id}` used unexpected payload namespace: {other:?}"
            ),
        };
        assert_eq!(actual_namespace, expected_namespace);
    }
}
