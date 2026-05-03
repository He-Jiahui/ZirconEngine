use std::collections::BTreeMap;

use serde_json::json;
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
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, document_pane, BuildExportPaneViewData,
    BuildExportTargetViewData, ModulePluginStatusViewData, ModulePluginsPaneViewData,
    PaneActionPresentation, PaneEmptyStatePresentation, PanePayload, PanePayloadBuildContext,
    PanePresentation, PaneShellPresentation,
};
use crate::ui::workbench::layout::{
    ActivityWindowId, DocumentNode, MainHostPageLayout, MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, InspectorSnapshot,
    ProjectOverviewSnapshot, SceneEntry, WelcomePaneSnapshot, WorkbenchSnapshot,
};
use crate::ui::workbench::startup::EditorSessionMode;
use crate::ui::workbench::view::{
    PaneBodySpec, PanePayloadKind, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance,
    ViewInstanceId,
};

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
    pane_descriptor(descriptor_id)
        .pane_template
        .map(|template| template.body)
        .unwrap_or_else(|| panic!("missing pane body spec for `{descriptor_id}`"))
}

fn pane_descriptor(descriptor_id: &str) -> ViewDescriptor {
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

fn editor_data_fixture() -> EditorDataSnapshot {
    EditorDataSnapshot {
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
    }
}

fn chrome_fixture() -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: editor_data_fixture().scene_entries,
        inspector: editor_data_fixture().inspector,
        status_line: editor_data_fixture().status_line,
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

fn module_plugins_fixture() -> ModulePluginsPaneViewData {
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
            feature_action_id: "Plugin.Feature.Enable.physics.physics.raycast_queries".into(),
            diagnostics: "".into(),
            primary_action_label: "Disable".into(),
            primary_action_id: "Plugin.Disable.physics".into(),
            packaging_action_label: "Cycle linked".into(),
            packaging_action_id: "Plugin.Packaging.Next.physics".into(),
            target_modes_action_label: "Cycle targets".into(),
            target_modes_action_id: "Plugin.TargetModes.Next.physics".into(),
            unload_action_label: "Unload".into(),
            unload_action_id: "Plugin.Unload.physics".into(),
            hot_reload_action_label: "Hot Reload".into(),
            hot_reload_action_id: "Plugin.HotReload.physics".into(),
        }]),
        diagnostics: "plugin catalog ready".into(),
    }
}

fn build_export_fixture() -> BuildExportPaneViewData {
    BuildExportPaneViewData {
        targets: crate::ui::layouts::common::model_rc(vec![BuildExportTargetViewData {
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
    }
}

#[test]
fn pane_payload_builders_emit_stable_body_metadata_for_first_wave_views() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let module_plugins = module_plugins_fixture();
    let build_export = build_export_fixture();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_module_plugins(&module_plugins)
        .with_build_export(&build_export);

    let cases = [
        (
            "editor.console",
            "pane.console.body",
            PanePayloadKind::ConsoleV1,
        ),
        (
            "editor.inspector",
            "pane.inspector.body",
            PanePayloadKind::InspectorV1,
        ),
        (
            "editor.hierarchy",
            "pane.hierarchy.body",
            PanePayloadKind::HierarchyV1,
        ),
        (
            "editor.animation_sequence",
            "pane.animation.sequence.body",
            PanePayloadKind::AnimationSequenceV1,
        ),
        (
            "editor.animation_graph",
            "pane.animation.graph.body",
            PanePayloadKind::AnimationGraphV1,
        ),
        (
            "editor.runtime_diagnostics",
            "pane.runtime.diagnostics.body",
            PanePayloadKind::RuntimeDiagnosticsV1,
        ),
        (
            "editor.module_plugins",
            "pane.module_plugins.body",
            PanePayloadKind::ModulePluginsV1,
        ),
        (
            "editor.build_export_desktop",
            "pane.build_export_desktop.body",
            PanePayloadKind::BuildExportV1,
        ),
    ];

    for (descriptor_id, document_id, payload_kind) in cases {
        let spec = pane_body_spec(descriptor_id);
        let body = build_pane_body_presentation(&spec, &context);

        assert_eq!(body.document_id, document_id);
        assert_eq!(body.payload_kind, payload_kind);
        assert_eq!(body.payload_kind, spec.payload_kind);
        assert_eq!(body.route_namespace, spec.route_namespace);
        assert_eq!(body.interaction_mode, spec.interaction_mode);

        match (descriptor_id, body.payload) {
            ("editor.console", PanePayload::ConsoleV1(payload)) => {
                assert_eq!(payload.status_text, "Console ready");
            }
            ("editor.inspector", PanePayload::InspectorV1(payload)) => {
                assert_eq!(payload.node_id, 7);
                assert_eq!(payload.name, "Root");
                assert_eq!(payload.translation, ["1.0", "2.0", "3.0"]);
            }
            ("editor.hierarchy", PanePayload::HierarchyV1(payload)) => {
                assert_eq!(payload.nodes.len(), 2);
                assert_eq!(payload.nodes[0].node_id, 7);
                assert_eq!(payload.nodes[0].name, "Root");
                assert!(payload.nodes[0].selected);
            }
            ("editor.animation_sequence", PanePayload::AnimationSequenceV1(payload)) => {
                assert_eq!(payload.asset_path, "res://animations/hero.anim");
                assert_eq!(payload.timeline_start_frame, 5);
                assert_eq!(payload.timeline_end_frame, 42);
                assert_eq!(payload.track_items, vec!["Root/Hero:Transform.position"]);
            }
            ("editor.animation_graph", PanePayload::AnimationGraphV1(payload)) => {
                assert_eq!(payload.asset_path, "res://animations/hero.anim");
                assert_eq!(payload.node_items, vec!["Blend"]);
                assert_eq!(payload.state_items, vec!["Idle", "Run"]);
                assert_eq!(payload.transition_items, vec!["Idle -> Run"]);
            }
            ("editor.runtime_diagnostics", PanePayload::RuntimeDiagnosticsV1(payload)) => {
                assert_eq!(payload.summary, "3 runtime systems available");
                assert_eq!(
                    payload.render_status,
                    "Render: wgpu-test (3 viewports, 11 frames)"
                );
                assert_eq!(payload.physics_status, "Physics: jolt (Ready, 120 Hz)");
                assert_eq!(
                    payload.animation_status,
                    "Animation: enabled (graphs on, state machines on)"
                );
                assert!(payload
                    .detail_items
                    .contains(&"Virtual Geometry Debug: available".to_string()));
                assert!(payload
                    .detail_items
                    .contains(&"Hybrid GI active probes: 4".to_string()));
            }
            ("editor.module_plugins", PanePayload::ModulePluginsV1(payload)) => {
                assert_eq!(payload.diagnostics, "plugin catalog ready");
                assert_eq!(payload.plugins.len(), 1);
                assert_eq!(payload.plugins[0].plugin_id, "physics");
                assert_eq!(payload.plugins[0].display_name, "Physics");
                assert!(payload.plugins[0].enabled);
                assert_eq!(
                    payload.plugins[0].optional_features,
                    "Ray Cast Queries [ready]"
                );
                assert_eq!(payload.plugins[0].feature_action_label, "Enable Feature");
                assert_eq!(
                    payload.plugins[0].feature_action_id,
                    "Plugin.Feature.Enable.physics.physics.raycast_queries"
                );
                assert_eq!(payload.plugins[0].primary_action_label, "Disable");
                assert_eq!(
                    payload.plugins[0].primary_action_id,
                    "Plugin.Disable.physics"
                );
                assert_eq!(
                    payload.plugins[0].packaging_action_id,
                    "Plugin.Packaging.Next.physics"
                );
                assert_eq!(
                    payload.plugins[0].target_modes_action_id,
                    "Plugin.TargetModes.Next.physics"
                );
                assert_eq!(payload.plugins[0].unload_action_label, "Unload");
                assert_eq!(payload.plugins[0].unload_action_id, "Plugin.Unload.physics");
                assert_eq!(payload.plugins[0].hot_reload_action_label, "Hot Reload");
                assert_eq!(
                    payload.plugins[0].hot_reload_action_id,
                    "Plugin.HotReload.physics"
                );
            }
            ("editor.build_export_desktop", PanePayload::BuildExportV1(payload)) => {
                assert_eq!(payload.diagnostics, "export catalog ready");
                assert_eq!(payload.targets.len(), 1);
                assert_eq!(payload.targets[0].platform, "Windows");
                assert_eq!(payload.targets[0].status, "Ready");
                assert_eq!(payload.targets[0].native_dynamic_packages, "1");
            }
            (unexpected_id, unexpected_payload) => panic!(
                "builder for `{unexpected_id}` produced unexpected payload {unexpected_payload:?}"
            ),
        }
    }
}

#[test]
fn pane_presentation_keeps_shell_and_body_split_without_erasing_payload_type() {
    let empty_state = PaneEmptyStatePresentation {
        title: "No Console".to_string(),
        body: "Nothing has been written yet.".to_string(),
        primary_action: Some(PaneActionPresentation {
            label: "Open".to_string(),
            action_id: "OpenConsole".to_string(),
        }),
        secondary_action: Some(PaneActionPresentation {
            label: "Dismiss".to_string(),
            action_id: "DismissConsole".to_string(),
        }),
        secondary_hint: "Wait for editor output".to_string(),
    };
    let shell = PaneShellPresentation::new(
        "Console",
        "console",
        "Task Output",
        "Console ready",
        Some(empty_state),
        false,
        blank_viewport_chrome(),
    );
    let chrome = chrome_fixture();
    let context = PanePayloadBuildContext::new(&chrome);
    let body = build_pane_body_presentation(&pane_body_spec("editor.console"), &context);
    let presentation = PanePresentation::new(shell.clone(), body.clone());

    assert_eq!(presentation.shell.title, "Console");
    assert_eq!(presentation.shell.icon_key, "console");
    assert_eq!(presentation.shell.subtitle, "Task Output");
    assert_eq!(presentation.shell.info, "Console ready");
    assert!(!presentation.shell.show_toolbar);
    assert_eq!(presentation.shell.viewport.tool, "");
    assert_eq!(
        presentation
            .shell
            .empty_state
            .as_ref()
            .and_then(|state| state.primary_action.as_ref())
            .map(|action| action.label.as_str()),
        Some("Open")
    );
    assert_eq!(
        presentation
            .shell
            .empty_state
            .as_ref()
            .map(|state| state.secondary_hint.as_str()),
        Some("Wait for editor output")
    );
    assert_eq!(presentation.body.document_id, "pane.console.body");
    match presentation.body.payload {
        PanePayload::ConsoleV1(payload) => assert_eq!(payload.status_text, "Console ready"),
        unexpected => panic!("expected console payload, found {unexpected:?}"),
    }
}

#[test]
fn document_pane_projects_first_wave_pane_presentations_alongside_legacy_data() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let cases = [
        ("editor.console", "pane.console.body"),
        ("editor.inspector", "pane.inspector.body"),
        ("editor.hierarchy", "pane.hierarchy.body"),
        ("editor.animation_sequence", "pane.animation.sequence.body"),
        ("editor.animation_graph", "pane.animation.graph.body"),
        (
            "editor.runtime_diagnostics",
            "pane.runtime.diagnostics.body",
        ),
        ("editor.module_plugins", "pane.module_plugins.body"),
        (
            "editor.build_export_desktop",
            "pane.build_export_desktop.body",
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
        );
        let model = WorkbenchViewModel::build(&chrome);
        let animation_panes = if descriptor_id.starts_with("editor.animation_") {
            BTreeMap::from([(instance_id.0.clone(), animation_fixture())])
        } else {
            BTreeMap::new()
        };
        let runtime_diagnostics = runtime_diagnostics_fixture();

        let pane = document_pane(
            &model,
            &chrome,
            &BTreeMap::new(),
            &animation_panes,
            Some(&runtime_diagnostics),
            &crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData::default(
            ),
            &crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData::default(),
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
                }
                unexpected => panic!("expected runtime diagnostics payload, found {unexpected:?}"),
            }
        }
    }
}
