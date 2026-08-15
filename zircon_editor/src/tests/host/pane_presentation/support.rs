use std::collections::BTreeMap;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::core::diagnostics::{
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, RuntimeAnimationDiagnostics,
    RuntimeDiagnosticsSnapshot, RuntimePhysicsBackendDiagnostics, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
};
use zircon_runtime::core::framework::animation::AnimationPlaybackSettings;
use zircon_runtime::core::framework::render::{RenderCapabilitySummary, RenderStats};
use zircon_runtime::foundation::{
    FOUNDATION_MODULE_NAME, module_descriptor as foundation_module_descriptor,
};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiFrame, UiLayoutEngineCapability, UiLayoutEngineFamily, UiLayoutEngineRequest,
        UiLayoutEngineSelection, UiLayoutEngineSelectionReport,
    },
    surface::{
        UiCanvasLayerGroup, UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind,
        UiRenderDebugStats, UiSurfaceDebugCaptureContext, UiSurfaceDebugSnapshot,
        UiWidgetReflectorNode,
    },
    tree::{UiInputPolicy, UiVisibility},
};

use crate::scene::viewport::SceneViewportChromeSettings;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::host::EditorManager;
use crate::ui::host::module::{self, EDITOR_MANAGER_NAME, module_descriptor};
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, BuildExportTargetViewData, ModulePluginStatusViewData,
    ModulePluginsPaneViewData,
};
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, ProjectOverviewSnapshot, SceneEntries,
    SceneEntry, WelcomePaneSnapshot, WorkbenchSnapshot,
};
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::view::{PaneBodySpec, ViewDescriptor, ViewDescriptorId};

pub(super) fn editor_runtime() -> CoreRuntime {
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

pub(super) fn pane_body_spec(descriptor_id: &str) -> PaneBodySpec {
    pane_descriptor(descriptor_id)
        .pane_template
        .map(|template| template.body)
        .unwrap_or_else(|| panic!("missing pane body spec for `{descriptor_id}`"))
}

pub(super) fn pane_descriptor(descriptor_id: &str) -> ViewDescriptor {
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
        .unwrap_or_else(|| panic!("missing descriptor for `{descriptor_id}`"))
}

pub(super) fn editor_data_fixture() -> EditorDataSnapshot {
    EditorDataSnapshot {
        scene_entries: SceneEntries::from_entries(
            vec![
                SceneEntry {
                    id: 7,
                    name: "Root".to_string(),
                    depth: 0,
                },
                SceneEntry {
                    id: 8,
                    name: "Camera".to_string(),
                    depth: 1,
                },
            ],
            [7],
        ),
        inspector: Some(InspectorSnapshot {
            id: 7,
            name: "Root".to_string(),
            parent: "Scene".to_string(),
            translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
            scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
            plugin_components: Vec::new(),
        }),
        status_line: "Console ready".to_string(),
        console_output: "Console ready".into(),
        status_task_progress: None,
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportChromeSettings::default(),
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
        bridge_diagnostics: Default::default(),
    }
}

pub(super) fn editor_data_with_drawer_fixture() -> EditorDataSnapshot {
    let mut data = editor_data_fixture();
    if let Some(inspector) = &mut data.inspector {
        inspector
            .plugin_components
            .push(InspectorPluginComponentSnapshot {
                component_id: "weather.Component.CloudLayer".to_string(),
                display_name: "Cloud Layer".to_string(),
                plugin_id: "weather".to_string(),
                customization_available: true,
                customization_ui_document: Some(
                    "asset://weather/editor/cloud_layer.inspector.zui".to_string(),
                ),
                customization_controller: Some(
                    "weather.editor.CloudLayerInspectorController".to_string(),
                ),
                customization_template_id: Some("weather.cloud_layer.inspector".to_string()),
                customization_data_root: Some(
                    "inspector.plugin_components.weather.Component.CloudLayer".to_string(),
                ),
                customization_bindings: vec!["weather.cloud_layer.refresh".to_string()],
                diagnostic: None,
                properties: Vec::new(),
            });
    }
    data
}

pub(super) fn chrome_fixture() -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        focused_document_kind: None,
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: editor_data_fixture().scene_entries,
        inspector: editor_data_fixture().inspector,
        status_line: editor_data_fixture().status_line,
        console_output: editor_data_fixture().console_output,
        status_task_progress: editor_data_fixture().status_task_progress,
        hovered_axis: editor_data_fixture().hovered_axis,
        viewport_size: editor_data_fixture().viewport_size,
        scene_viewport_settings: editor_data_fixture().scene_viewport_settings,
        mesh_import_path: editor_data_fixture().mesh_import_path,
        project_overview: editor_data_fixture().project_overview,
        asset_activity: editor_data_fixture().asset_activity,
        asset_browser: editor_data_fixture().asset_browser,
        project_path: editor_data_fixture().project_path,
        session_mode: editor_data_fixture().session_mode,
        welcome: editor_data_fixture().welcome,
        project_open: editor_data_fixture().project_open,
        can_undo: editor_data_fixture().can_undo,
        can_redo: editor_data_fixture().can_redo,
        menu_overflow_mode: Default::default(),
    }
}

pub(super) fn animation_fixture() -> AnimationEditorPanePresentation {
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

pub(super) fn runtime_diagnostics_fixture() -> RuntimeDiagnosticsSnapshot {
    let profile = ProfileSnapshot {
        active: true,
        feature_enabled: true,
        frames: vec![ProfileFrameSnapshot {
            stream: "editor".to_string(),
            name: "retained_host_tick".to_string(),
            frame_index: 0,
            start_us: 0,
            duration_us: 18_000,
            budget_ms: 16.67,
            over_budget: true,
        }],
        spans: vec![ProfileSpanSnapshot {
            id: 1,
            parent_id: None,
            frame_index: Some(0),
            stream: "editor".to_string(),
            category: "retained_host".to_string(),
            name: "recompute_if_dirty".to_string(),
            path: "editor/retained_host:recompute_if_dirty".to_string(),
            start_us: 1_000,
            duration_us: 12_000,
            depth: 0,
        }],
        counters: vec![zircon_runtime::core::diagnostics::ProfileCounterSnapshot {
            stream: "editor".to_string(),
            name: "ui.scenario.hover.slow_path_rebuild".to_string(),
            value: 1.0,
            timestamp_us: 2_000,
            frame_index: Some(0),
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
            backend_status: Some(RuntimePhysicsBackendDiagnostics {
                requested_backend: "jolt".to_string(),
                active_backend: Some("jolt".to_string()),
                state: "ready".to_string(),
                detail: None,
                simulation_mode: "simulate".to_string(),
                feature_gate: Some("backend-jolt".to_string()),
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

pub(super) fn active_ui_debug_snapshot_fixture() -> UiSurfaceDebugSnapshot {
    UiSurfaceDebugSnapshot {
        capture: UiSurfaceDebugCaptureContext {
            surface_name: Some("Runtime Diagnostics fixture".to_string()),
            selected_node: Some(UiNodeId::new(2)),
            ..UiSurfaceDebugCaptureContext::default()
        },
        tree_id: UiTreeId::new("editor.runtime_diagnostics.active_debug"),
        roots: vec![UiNodeId::new(1)],
        nodes: vec![
            UiWidgetReflectorNode {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("runtime/root"),
                parent: None,
                children: vec![UiNodeId::new(2)],
                frame: UiFrame::new(0.0, 0.0, 120.0, 80.0),
                clip_frame: UiFrame::new(0.0, 0.0, 120.0, 80.0),
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Ignore,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                control_id: Some("RuntimeDiagnosticsRoot".to_string()),
                slot: None,
                render_command_count: 1,
                hit_entry_count: 0,
                hit_cell_count: 0,
            },
            UiWidgetReflectorNode {
                node_id: UiNodeId::new(2),
                node_path: UiNodePath::new("runtime/root/live_button"),
                parent: Some(UiNodeId::new(1)),
                children: Vec::new(),
                frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
                clip_frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
                z_index: 1,
                paint_order: 1,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: true,
                control_id: Some("LiveDebugButton".to_string()),
                slot: None,
                render_command_count: 2,
                hit_entry_count: 1,
                hit_cell_count: 1,
            },
        ],
        render: UiRenderDebugStats {
            command_count: 3,
            estimated_draw_calls: 3,
            ..UiRenderDebugStats::default()
        },
        layout_engine_report: layout_engine_report_fixture(),
        canvas_layers: vec![UiCanvasLayerGroup {
            parent_id: UiNodeId::new(1),
            layer_index: 0,
            z_order: 1,
            child_ids: vec![UiNodeId::new(2)],
        }],
        overlay_primitives: vec![UiDebugOverlayPrimitive {
            kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
            node_id: Some(UiNodeId::new(2)),
            frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
            label: Some("live".to_string()),
            severity: None,
        }],
        ..UiSurfaceDebugSnapshot::default()
    }
}

fn layout_engine_report_fixture() -> UiLayoutEngineSelectionReport {
    let taffy = UiLayoutEngineCapability::taffy_flex_grid_wrap_block();
    let zircon = UiLayoutEngineCapability::zircon();
    UiLayoutEngineSelectionReport::from_selections(vec![
        UiLayoutEngineSelection::select(
            &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Flex),
            &taffy,
            &zircon,
        )
        .with_node_id(UiNodeId::new(1)),
        UiLayoutEngineSelection::select(
            &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Overlay),
            &taffy,
            &zircon,
        )
        .with_node_id(UiNodeId::new(2)),
    ])
}

pub(super) fn module_plugins_fixture() -> ModulePluginsPaneViewData {
    ModulePluginsPaneViewData {
        plugins: crate::ui::layouts::common::model_rc(vec![ModulePluginStatusViewData {
            plugin_id: "physics".into(),
            display_name: "Physics".into(),
            package_source: "builtin".into(),
            load_state: "loaded".into(),
            enabled: true,
            required: false,
            target_modes: "editor, runtime".into(),
            packaging: "linked".into(),
            runtime_crate: "zircon_plugins_physics_runtime".into(),
            editor_crate: "zircon_plugins_physics_editor".into(),
            runtime_capabilities: "simulation".into(),
            editor_capabilities: "inspector".into(),
            optional_features: "Ray Cast Queries [ready]".into(),
            feature_action_label: "Enable Feature".into(),
            feature_action_id: "workbench.plugin.feature.enable.physics.physics.raycast_queries"
                .into(),
            diagnostics: "".into(),
            primary_action_label: "Disable".into(),
            primary_action_id: "workbench.plugin.disable.physics".into(),
            packaging_action_label: "Cycle linked".into(),
            packaging_action_id: "workbench.plugin.packaging.next.physics".into(),
            target_modes_action_label: "Cycle targets".into(),
            target_modes_action_id: "workbench.plugin.target_modes.next.physics".into(),
            unload_action_label: "Unload".into(),
            unload_action_id: "workbench.plugin.unload.physics".into(),
            hot_reload_action_label: "Hot Reload".into(),
            hot_reload_action_id: "workbench.plugin.hot_reload.physics".into(),
        }]),
        diagnostics: "plugin catalog ready".into(),
    }
}

pub(super) fn build_export_fixture() -> BuildExportPaneViewData {
    BuildExportPaneViewData {
        targets: crate::ui::layouts::common::model_rc(vec![BuildExportTargetViewData {
            preset_name: "desktop_windows".into(),
            profile_name: "desktop_windows".into(),
            platform: "Windows".into(),
            target_mode: "ClientRuntime".into(),
            strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
            status: "Ready".into(),
            enabled_plugins: "3".into(),
            linked_runtime_crates: "2".into(),
            native_dynamic_packages: "1".into(),
            generated_files: "5".into(),
            diagnostics: "native plugin package ready".into(),
            fatal: false,
        }]),
        diagnostics: "export catalog ready".into(),
        ..BuildExportPaneViewData::default()
    }
}
