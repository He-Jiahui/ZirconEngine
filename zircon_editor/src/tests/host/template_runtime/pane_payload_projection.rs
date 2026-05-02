use std::collections::BTreeMap;

use zircon_runtime::core::diagnostics::{
    RuntimeAnimationDiagnostics, RuntimeDiagnosticsSnapshot, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
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

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, PanePayloadBuildContext,
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
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics);
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
}

#[test]
fn editor_ui_host_runtime_exposes_hybrid_slot_anchors_in_host_projection() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let context = PanePayloadBuildContext::new(&chrome).with_animation_pane(&animation);
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
