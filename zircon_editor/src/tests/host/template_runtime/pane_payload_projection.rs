use std::collections::BTreeMap;

use zircon_runtime::core::diagnostics::{
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, RuntimeAnimationDiagnostics,
    RuntimeDiagnosticsSnapshot, RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
};
use zircon_runtime::core::framework::animation::AnimationPlaybackSettings;
use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsSimulationMode,
};
use zircon_runtime::core::framework::render::{RenderCapabilitySummary, RenderStats};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{
        UiRenderDebugStats, UiSurfaceDebugCaptureContext, UiSurfaceDebugSnapshot,
        UiWidgetReflectorNode,
    },
    tree::{UiInputPolicy, UiVisibility},
};

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, BuildExportPaneViewData, ModulePluginsPaneViewData,
    PanePayloadBuildContext,
};
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, InspectorSnapshot, ProjectOverviewSnapshot,
    SceneEntry, WelcomePaneSnapshot, WorkbenchSnapshot,
};
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::view::{PaneBodySpec, ViewDescriptorId};
use toml::Value;

fn editor_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn pane_body_spec(descriptor_id: &str) -> PaneBodySpec {
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
        .and_then(|descriptor| descriptor.pane_template.map(|template| template.body))
        .unwrap_or_else(|| panic!("missing pane body spec for `{descriptor_id}`"))
}

fn chrome_fixture() -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: vec![
            SceneEntry {
                id: 7,
                name: "Root".to_string(),
                depth: 0,
                selected: true,
            },
            SceneEntry {
                id: 8,
                name: "Camera".to_string(),
                depth: 1,
                selected: false,
            },
        ],
        inspector: Some(InspectorSnapshot {
            id: 7,
            name: "Root".to_string(),
            parent: "Scene".to_string(),
            translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
            plugin_components: Vec::new(),
        }),
        status_line: "Console ready".to_string(),
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportSettings::default(),
        mesh_import_path: String::new(),
        project_overview: ProjectOverviewSnapshot::default(),
        asset_activity: AssetWorkspaceSnapshot::default(),
        asset_browser: AssetWorkspaceSnapshot::default(),
        project_path: "sandbox-project".to_string(),
        session_mode: EditorSessionMode::Project,
        welcome: WelcomePaneSnapshot::default(),
        project_open: true,
        can_undo: true,
        can_redo: false,
        menu_overflow_mode: Default::default(),
    }
}

fn animation_fixture() -> AnimationEditorPanePresentation {
    AnimationEditorPanePresentation {
        mode: "sequence".to_string(),
        asset_path: "res://animations/hero.anim".to_string(),
        status: "Ready".to_string(),
        selection_summary: "Track Root/Hero selected".to_string(),
        current_frame: 12,
        timeline_start_frame: 5,
        timeline_end_frame: 42,
        playback_label: "Paused".to_string(),
        track_items: vec!["Root/Hero:Transform.position".to_string()],
        parameter_items: vec!["speed".to_string()],
        node_items: vec!["Blend".to_string()],
        state_items: vec!["Idle".to_string(), "Run".to_string()],
        transition_items: vec!["Idle -> Run".to_string()],
    }
}

fn runtime_diagnostics_fixture() -> RuntimeDiagnosticsSnapshot {
    let profile = ProfileSnapshot {
        active: true,
        feature_enabled: true,
        frames: vec![ProfileFrameSnapshot {
            stream: "editor".to_string(),
            name: "retained_host_tick".to_string(),
            frame_index: 42,
            start_us: 1_000,
            duration_us: 17_250,
            budget_ms: 16.67,
            over_budget: true,
        }],
        spans: vec![ProfileSpanSnapshot {
            id: 7,
            parent_id: None,
            frame_index: Some(42),
            stream: "editor".to_string(),
            category: "retained_host".to_string(),
            name: "present_frame".to_string(),
            path: "editor/retained_host:present_frame".to_string(),
            start_us: 2_000,
            duration_us: 8_500,
            depth: 0,
        }],
        ..ProfileSnapshot::default()
    };

    RuntimeDiagnosticsSnapshot {
        render: RuntimeRenderDiagnostics {
            available: true,
            stats: Some(RenderStats {
                active_viewports: 3,
                submitted_frames: 11,
                last_hybrid_gi_active_probe_count: 4,
                capabilities: RenderCapabilitySummary {
                    backend_name: "wgpu-test".to_string(),
                    virtual_geometry_supported: true,
                    hybrid_global_illumination_supported: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            virtual_geometry_debug_available: true,
            error: None,
        },
        physics: RuntimePhysicsDiagnostics {
            available: true,
            backend_name: Some("jolt".to_string()),
            backend_status: Some(PhysicsBackendStatus {
                requested_backend: "jolt".to_string(),
                active_backend: Some("jolt".to_string()),
                state: PhysicsBackendState::Ready,
                detail: None,
                simulation_mode: PhysicsSimulationMode::Simulate,
                feature_gate: Some("jolt".to_string()),
            }),
            fixed_hz: Some(120),
            error: None,
        },
        animation: RuntimeAnimationDiagnostics {
            available: true,
            playback_settings: Some(AnimationPlaybackSettings {
                enabled: true,
                property_tracks: true,
                skeletal_clips: true,
                graphs: true,
                state_machines: true,
            }),
            error: None,
        },
        store: Default::default(),
        profile,
    }
}

fn active_ui_debug_snapshot_fixture() -> UiSurfaceDebugSnapshot {
    UiSurfaceDebugSnapshot {
        capture: UiSurfaceDebugCaptureContext {
            selected_node: Some(UiNodeId::new(2)),
            ..UiSurfaceDebugCaptureContext::default()
        },
        tree_id: UiTreeId::new("editor.runtime_diagnostics.projection_debug"),
        roots: vec![UiNodeId::new(1)],
        nodes: vec![
            UiWidgetReflectorNode {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("runtime/projection"),
                parent: None,
                children: vec![UiNodeId::new(2)],
                frame: UiFrame::new(0.0, 0.0, 160.0, 80.0),
                clip_frame: UiFrame::new(0.0, 0.0, 160.0, 80.0),
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Ignore,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                control_id: Some("RuntimeDiagnosticsProjectionRoot".to_string()),
                render_command_count: 1,
                hit_entry_count: 0,
                hit_cell_count: 0,
            },
            UiWidgetReflectorNode {
                node_id: UiNodeId::new(2),
                node_path: UiNodePath::new("runtime/projection/live_label"),
                parent: Some(UiNodeId::new(1)),
                children: Vec::new(),
                frame: UiFrame::new(8.0, 12.0, 120.0, 18.0),
                clip_frame: UiFrame::new(8.0, 12.0, 120.0, 18.0),
                z_index: 1,
                paint_order: 1,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                control_id: Some("LiveProjectionLabel".to_string()),
                render_command_count: 1,
                hit_entry_count: 1,
                hit_cell_count: 1,
            },
        ],
        render: UiRenderDebugStats {
            command_count: 2,
            estimated_draw_calls: 2,
            ..UiRenderDebugStats::default()
        },
        ..UiSurfaceDebugSnapshot::default()
    }
}

#[test]
fn editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let active_snapshot = active_ui_debug_snapshot_fixture();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_active_ui_debug_snapshot(&active_snapshot)
        .with_module_plugins(&module_plugins)
        .with_build_export(&build_export);
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let console = build_pane_body_presentation(&pane_body_spec("editor.console"), &context);
    let console_projection = runtime.project_pane_body(&console).unwrap();
    assert_eq!(
        console_projection.root.attributes.get("pane_payload_kind"),
        Some(&Value::String("ConsoleV1".to_string()))
    );
    assert_eq!(
        console_projection
            .root
            .attributes
            .get("pane_route_namespace"),
        Some(&Value::String("Dock".to_string()))
    );
    assert_eq!(
        console_projection
            .root
            .attributes
            .get("payload_status_text"),
        Some(&Value::String("Console ready".to_string()))
    );

    let inspector = build_pane_body_presentation(&pane_body_spec("editor.inspector"), &context);
    let inspector_projection = runtime.project_pane_body(&inspector).unwrap();
    assert_eq!(
        inspector_projection
            .root
            .attributes
            .get("pane_payload_kind"),
        Some(&Value::String("InspectorV1".to_string()))
    );
    assert_eq!(
        inspector_projection.root.attributes.get("payload_name"),
        Some(&Value::String("Root".to_string()))
    );
    assert_eq!(
        inspector_projection
            .root
            .attributes
            .get("payload_delete_enabled"),
        Some(&Value::Boolean(true))
    );

    let diagnostics =
        build_pane_body_presentation(&pane_body_spec("editor.runtime_diagnostics"), &context);
    let diagnostics_projection = runtime.project_pane_body(&diagnostics).unwrap();
    assert_eq!(
        diagnostics_projection
            .root
            .attributes
            .get("pane_payload_kind"),
        Some(&Value::String("RuntimeDiagnosticsV1".to_string()))
    );
    assert_eq!(
        diagnostics_projection
            .root
            .attributes
            .get("payload_summary"),
        Some(&Value::String("3 runtime systems available".to_string()))
    );
    assert_eq!(
        diagnostics_projection
            .root
            .attributes
            .get("payload_render_status"),
        Some(&Value::String(
            "Render: wgpu-test (3 viewports, 11 frames)".to_string()
        ))
    );
    assert_eq!(
        diagnostics_projection
            .root
            .attributes
            .get("payload_ui_debug_reflector_summary"),
        Some(&Value::String(
            "UI Debug Reflector: 2 nodes, 2 commands, schema v1".to_string()
        ))
    );
    assert!(diagnostics_projection
        .root
        .attributes
        .get("payload_ui_debug_reflector_export_status")
        .and_then(Value::as_str)
        .is_some_and(|text| text.contains("JSON export ready")));
    assert!(diagnostics_projection
        .root
        .attributes
        .get("payload_ui_debug_reflector_details")
        .and_then(Value::as_array)
        .is_some_and(|details| details.iter().any(|detail| {
            detail
                .as_str()
                .is_some_and(|text| text.contains("Selected: runtime/projection/live_label"))
        })));

    let timeline =
        build_pane_body_presentation(&pane_body_spec("editor.performance_timeline"), &context);
    let timeline_projection = runtime.project_pane_body(&timeline).unwrap();
    assert_eq!(
        timeline_projection.root.attributes.get("pane_payload_kind"),
        Some(&Value::String("PerformanceTimelineV1".to_string()))
    );
    assert_eq!(
        timeline_projection.root.attributes.get("payload_summary"),
        Some(&Value::String(
            "Profiling active: 1 frame, 1 span, 0 counters".to_string()
        ))
    );
    assert!(timeline_projection
        .root
        .attributes
        .get("payload_frame_rows")
        .and_then(Value::as_array)
        .is_some_and(|rows| rows.iter().any(|row| {
            row.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == "retained_host_tick")
        })));
    assert!(timeline_projection
        .root
        .attributes
        .get("payload_hotspot_rows")
        .and_then(Value::as_array)
        .is_some_and(|rows| rows.iter().any(|row| {
            row.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == "present_frame")
        })));
}

#[test]
fn editor_ui_host_runtime_exposes_hybrid_slot_anchors_in_host_projection() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_module_plugins(&module_plugins)
        .with_build_export(&build_export);
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let cases = [
        (
            "editor.hierarchy",
            "HierarchyTreeSlotAnchor",
            "hierarchy_tree_slot",
        ),
        (
            "editor.animation_sequence",
            "AnimationTimelineSlotAnchor",
            "animation_timeline_slot",
        ),
        (
            "editor.animation_graph",
            "AnimationGraphCanvasSlotAnchor",
            "animation_graph_canvas_slot",
        ),
        (
            "editor.performance_timeline",
            "PerformanceTimelineFrameListSlotAnchor",
            "performance_timeline_frame_list",
        ),
    ];

    for (descriptor_id, control_id, slot_name) in cases {
        let body = build_pane_body_presentation(&pane_body_spec(descriptor_id), &context);
        let projection = runtime.project_pane_body(&body).unwrap();
        let host_model = runtime.build_host_model(&projection).unwrap();
        let anchor = host_model
            .node_by_control_id(control_id)
            .unwrap_or_else(|| panic!("missing hybrid slot anchor `{control_id}`"));

        assert_eq!(
            anchor.attributes.get("slot_name"),
            Some(&Value::String(slot_name.to_string()))
        );
        assert_eq!(
            anchor.attributes.get("pane_interaction_mode"),
            Some(&Value::String("HybridNativeSlot".to_string()))
        );
        assert_eq!(
            host_model
                .node("root")
                .and_then(|root| root.attributes.get("pane_route_namespace")),
            Some(&Value::String(body.route_namespace.to_string()))
        );
    }
}

#[test]
fn editor_ui_host_runtime_preserves_hybrid_slot_anchors_in_surface_host_projection() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_module_plugins(&module_plugins)
        .with_build_export(&build_export);
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    for (descriptor_id, control_id) in [
        ("editor.animation_sequence", "AnimationTimelineSlotAnchor"),
        ("editor.animation_graph", "AnimationGraphCanvasSlotAnchor"),
        (
            "editor.performance_timeline",
            "PerformanceTimelineFrameListSlotAnchor",
        ),
    ] {
        let body = build_pane_body_presentation(&pane_body_spec(descriptor_id), &context);
        let projection = runtime.project_pane_body(&body).unwrap();
        let mut surface = runtime.build_shared_surface(&body.document_id).unwrap();
        surface
            .compute_layout(zircon_runtime_interface::ui::layout::UiSize::new(
                520.0, 300.0,
            ))
            .unwrap();
        let host_model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .unwrap();

        assert!(
            host_model.node_by_control_id(control_id).is_some(),
            "surface-backed host projection should preserve `{control_id}`"
        );
    }
}
