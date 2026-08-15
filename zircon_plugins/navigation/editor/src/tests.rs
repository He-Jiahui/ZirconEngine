use super::*;
use crate::bake_panel::{
    NavigationBakeAction, NavigationBakeBackend, NavigationBakePanel,
    NavigationBakePanelController, NavigationBakePhase, NavigationBakeProgress,
    NavigationBakeSelectedSubmitError, NavigationBakeSelectionError, NavigationBakeSurfaceRow,
};
use crate::overlay::{
    build_navigation_overlay, NavigationOverlayController, NavigationOverlayOptions,
    NavigationViewportGizmoSink,
};
use crate::runtime_mirror::{NavigationPieFrame, NavigationPieMirror, NavigationPieMirrorApply};
use std::sync::{Arc, Mutex};
use zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerHost;
use zircon_editor::{EditorRuntimeGateway, GatewayError};
use zircon_plugin_navigation_runtime::NavigationOverlayFrame;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavPathStatus, NavigationAgentDebugState,
    NavigationGizmoSnapshot, NavigationGizmoTriangle, AREA_JUMP, AREA_WALKABLE,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::render::SceneGizmoKind;
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeFrameV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

mod bake_panel_retained;
mod operation_command;
mod viewport_overlay_provider;

#[derive(Default)]
struct RecordingBakeBackend {
    requests: Vec<crate::NavigationBakeRequest>,
    reject: bool,
}

impl NavigationBakeBackend for RecordingBakeBackend {
    type Error = &'static str;

    fn submit(&mut self, request: crate::NavigationBakeRequest) -> Result<(), Self::Error> {
        self.requests.push(request);
        if self.reject {
            Err("bake queue unavailable")
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct RecordingOverlaySink {
    submissions: Vec<Option<SceneGizmoOverlayExtract>>,
}

struct NavigationMirrorRuntimeGateway {
    deliveries: Mutex<Vec<ZrRuntimePluginEventDeliveryV1>>,
}

impl NavigationMirrorRuntimeGateway {
    fn new(delivery: ZrRuntimePluginEventDeliveryV1) -> Self {
        Self {
            deliveries: Mutex::new(vec![delivery]),
        }
    }
}

impl EditorRuntimeGateway for NavigationMirrorRuntimeGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(42)
    }

    fn handle_event(&self, _event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        Ok(())
    }

    fn capture_frame(
        &self,
        _viewport: ZrRuntimeViewportHandle,
        _size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, GatewayError> {
        Ok(ZrRuntimeFrameV1::empty(1))
    }

    fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        assert_eq!(event_id, NAVIGATION_OVERLAY_FRAME_EVENT_ID);
        assert_eq!(payload_schema, NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA);
        Ok(Some(ZrRuntimePluginEventSubscriptionHandle::new(9)))
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        assert_eq!(subscription.raw(), 9);
        Ok(true)
    }

    fn drain_plugin_events(
        &self,
        _subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        Ok(std::mem::take(&mut *self.deliveries.lock().unwrap()))
    }

    fn submit_operation(
        &self,
        _request: zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
    ) -> Result<zircon_runtime_interface::ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

impl NavigationViewportGizmoSink for RecordingOverlaySink {
    fn replace_navigation_overlay(&mut self, overlay: Option<SceneGizmoOverlayExtract>) {
        self.submissions.push(overlay);
    }
}

#[test]
fn navigation_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&NAVIGATION_AUTHORING_CAPABILITY.to_string()));
    for view_id in [
        NAVIGATION_AUTHORING_VIEW_ID,
        NAVIGATION_AGENTS_VIEW_ID,
        NAVIGATION_BAKE_VIEW_ID,
        NAVIGATION_DEBUG_VIEW_ID,
    ] {
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == view_id));
    }
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == NAVIGATION_DRAWER_ID));
    for template_id in [
        NAVIGATION_TEMPLATE_ID,
        NAVIGATION_AGENTS_TEMPLATE_ID,
        NAVIGATION_BAKE_TEMPLATE_ID,
        NAVIGATION_DEBUG_TEMPLATE_ID,
        NAVIGATION_ASSET_TEMPLATE_ID,
        NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID,
    ] {
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == template_id));
    }
    for component_type in [
        NAV_MESH_SURFACE_COMPONENT_TYPE,
        NAV_MESH_MODIFIER_COMPONENT_TYPE,
        NAV_MESH_AGENT_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    ] {
        assert!(registration
            .extensions
            .inspector_customizations()
            .iter()
            .any(|customization| customization.target_type() == component_type));
    }
    for operation in [
        "view.navigation.surfaces.open",
        "view.navigation.agents_areas.open",
        "view.navigation.bake.open",
        "view.navigation.debug_gizmos.open",
        NAVIGATION_BAKE_SCENE_OPERATION,
        NAVIGATION_BAKE_SURFACE_OPERATION,
        NAVIGATION_CLEAR_SURFACE_OPERATION,
        NAVIGATION_OPEN_SETTINGS_OPERATION,
        NAVIGATION_TOGGLE_GIZMOS_OPERATION,
        NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION,
        NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION,
    ] {
        assert!(registration
            .extensions
            .command_ids()
            .any(|command_id| command_id.as_str() == operation));
    }
    assert!(registration
        .extensions
        .asset_type_contributions()
        .iter()
        .any(|contribution| contribution.asset_type().as_str() == "navigation.mesh"));
    assert!(registration
        .extensions
        .asset_type_contributions()
        .iter()
        .any(|contribution| contribution.asset_type().as_str() == "navigation.settings"));

    for document in navigation_editor_documents() {
        assert!(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(document)
                .exists(),
            "missing navigation editor document {document}"
        );
    }
}

#[test]
fn navigation_editor_sdk_declaration_and_capability_diagnostics_are_authoritative() {
    let plugin = editor_plugin();
    let declaration = crate::plugin::editor_plugin_declaration();
    assert_eq!(
        declaration.descriptor(),
        zircon_editor::EditorPlugin::descriptor(&plugin)
    );
    assert_eq!(declaration.package_manifest(), package_manifest());
    assert_eq!(
        declaration.mirrored_runtime_package_id(),
        Some("navigation")
    );

    let manager = zircon_editor::core::plugin::EditorPluginManager::from_plugins([(
        Arc::new(plugin) as Arc<dyn zircon_editor::EditorPlugin + Send + Sync>,
        zircon_plugin_navigation_runtime::package_manifest(),
    )])
    .expect("the navigation editor plugin should be admitted");
    let catalog = manager.catalog_snapshot();
    let missing = catalog.validate_capabilities(Vec::<String>::new());
    assert!(!missing.is_success());
    assert_eq!(missing.diagnostics.len(), EDITOR_CAPABILITIES.len());
    assert!(missing
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == "editor.capability.missing"));
    assert!(catalog
        .validate_capabilities(EDITOR_CAPABILITIES)
        .is_success());
}

#[test]
fn navigation_editor_consumer_is_manifest_projected_and_receives_pie_delivery() {
    let plugin = editor_plugin();
    let mirror = plugin.pie_mirror();
    let registration = plugin.registration_report();
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    let manifest = registration
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == "navigation.editor")
        .and_then(|module| module.event_consumers.first())
        .expect("navigation editor consumer manifest");
    assert_eq!(manifest.consumer_id, NAVIGATION_OVERLAY_CONSUMER_ID);
    assert_eq!(manifest.event_id, NAVIGATION_OVERLAY_FRAME_EVENT_ID);
    assert_eq!(
        manifest.payload_schema,
        NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA
    );

    let payload = NavigationOverlayFrame {
        owner_generation: 3,
        nav_mesh: NavigationGizmoSnapshot::default(),
        tick_report: NavAgentTickReport {
            scanned_agents: 5,
            moved_agents: 4,
            ..NavAgentTickReport::default()
        },
    };
    let delivery = ZrRuntimePluginEventDeliveryV1::new(
        42,
        ZrRuntimePluginEventSubscriptionHandle::new(9),
        NAVIGATION_OVERLAY_FRAME_EVENT_ID,
        NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
        1,
        serde_json::to_value(payload).unwrap(),
    );
    let host = EditorRuntimeEventConsumerHost::new(zircon_editor::EditorRuntimeGatewayHandle::new(
        Arc::new(NavigationMirrorRuntimeGateway::new(delivery)),
    ));
    host.register(registration.runtime_event_consumers).unwrap();
    host.begin_play_session(42, &[NAVIGATION_GIZMOS_CAPABILITY.to_string()])
        .unwrap();
    assert_eq!(host.pump().unwrap(), 1);
    assert_eq!(
        mirror.lock().unwrap().tick_report().unwrap().moved_agents,
        4
    );
    host.end_play_session(42).unwrap();
    assert!(mirror.lock().unwrap().tick_report().is_none());
}

fn navigation_editor_documents() -> &'static [&'static str] {
    &[
        "surfaces.zui",
        "agents_areas.zui",
        "bake.zui",
        "debug_gizmos.zui",
        "navmesh_asset.zui",
        "navigation_settings_asset.zui",
        "navmesh_surface.drawer.zui",
        "navmesh_modifier.drawer.zui",
        "navmesh_agent.drawer.zui",
        "navmesh_obstacle.drawer.zui",
        "navmesh_offmesh_link.drawer.zui",
    ]
}

#[test]
fn navigation_bake_panel_routes_commands_and_monotonic_progress() {
    let mut panel = NavigationBakePanel::default();
    let request = panel
        .submit(NavigationBakeAction::bake_selected_surface(42, true))
        .expect("idle panel accepts a bake request");
    assert_eq!(
        request.action.runtime_request().unwrap().surface_entity,
        Some(42)
    );
    assert!(request.action.runtime_request().unwrap().force_full_rebuild);
    assert_eq!(request.id, 1);
    assert_eq!(panel.phase(), NavigationBakePhase::Queued);
    assert!(panel.observe_progress(NavigationBakeProgress::new(
        request.id,
        NavigationBakePhase::Baking,
        3,
        10,
        "rasterizing tiles",
    )));
    assert!(!panel.observe_progress(NavigationBakeProgress::new(
        request.id,
        NavigationBakePhase::Baking,
        2,
        10,
        "stale worker update",
    )));
    assert!(!panel.observe_progress(NavigationBakeProgress::new(
        request.id,
        NavigationBakePhase::Queued,
        3,
        10,
        "phase rollback",
    )));
    assert!(!panel.observe_progress(NavigationBakeProgress::new(
        request.id,
        NavigationBakePhase::Baking,
        3,
        20,
        "fraction rollback",
    )));
    assert!(!panel.observe_progress(NavigationBakeProgress::new(
        request.id,
        NavigationBakePhase::Clearing,
        4,
        10,
        "wrong action phase",
    )));

    let report = NavMeshBakeReport {
        tiles: 10,
        baked_polygons: 128,
        ..NavMeshBakeReport::default()
    };
    assert!(panel.complete(request.id, Ok(report.clone())));
    assert_eq!(panel.phase(), NavigationBakePhase::Complete);
    assert_eq!(panel.last_report(), Some(&report));
    assert_eq!(panel.progress().fraction(), 1.0);
}

#[test]
fn navigation_bake_controller_submits_typed_requests_and_surfaces_backend_rejection() {
    let mut controller = NavigationBakePanelController::new(RecordingBakeBackend::default());
    let request = controller
        .submit(NavigationBakeAction::bake_scene(true))
        .expect("backend accepts request");
    assert_eq!(controller.backend().requests, [request.clone()]);
    assert!(request.action.runtime_request().unwrap().force_full_rebuild);

    controller.complete(
        request.id,
        Ok(NavMeshBakeReport {
            tiles: 1,
            ..NavMeshBakeReport::default()
        }),
    );
    controller.backend_mut().reject = true;
    let error = controller
        .submit(NavigationBakeAction::ClearSelectedSurface { entity: 9 })
        .expect_err("backend rejection must reach the panel");
    assert_eq!(error.to_string(), "bake queue unavailable");
    assert_eq!(controller.panel().phase(), NavigationBakePhase::Failed);
    assert_eq!(
        controller.panel().last_error(),
        Some("bake queue unavailable")
    );
}

#[test]
fn navigation_bake_selection_projects_stable_surface_entities_into_typed_actions() {
    let mut panel = NavigationBakePanel::default();
    panel.replace_surface_rows([
        NavigationBakeSurfaceRow::new(41, "Upper Deck"),
        NavigationBakeSurfaceRow::new(73, "Lower Deck"),
    ]);

    assert_eq!(panel.selected_surface_entity(), None);
    assert!(panel.select_surface(41));
    panel.set_force_full_rebuild(true);
    assert_eq!(
        panel.bake_selected_action(),
        Ok(NavigationBakeAction::bake_selected_surface(41, true))
    );

    assert!(panel.select_surface(73));
    assert_eq!(
        panel.clear_selected_action(),
        Ok(NavigationBakeAction::ClearSelectedSurface { entity: 73 })
    );
    assert_eq!(panel.selected_surface_entity(), Some(73));
}

#[test]
fn navigation_bake_selection_never_submits_without_a_current_surface() {
    let mut controller = NavigationBakePanelController::new(RecordingBakeBackend::default());
    controller.replace_surface_rows([NavigationBakeSurfaceRow::new(41, "Upper Deck")]);

    assert_eq!(
        controller.bake_selected(),
        Err(NavigationBakeSelectedSubmitError::Selection(
            NavigationBakeSelectionError::NoSurfaceSelected
        ))
    );
    assert_eq!(
        controller.clear_selected(),
        Err(NavigationBakeSelectedSubmitError::Selection(
            NavigationBakeSelectionError::NoSurfaceSelected
        ))
    );
    assert!(controller.backend().requests.is_empty());
}

#[test]
fn navigation_bake_selection_drops_removed_and_stale_surface_entities() {
    let mut panel = NavigationBakePanel::default();
    panel.replace_surface_rows([
        NavigationBakeSurfaceRow::new(41, "Upper Deck"),
        NavigationBakeSurfaceRow::new(73, "Lower Deck"),
    ]);
    assert!(panel.select_surface(41));

    panel.replace_surface_rows([NavigationBakeSurfaceRow::new(73, "Lower Deck")]);
    assert_eq!(panel.selected_surface_entity(), None);
    assert!(!panel.select_surface(999));
    assert_eq!(
        panel.clear_selected_action(),
        Err(NavigationBakeSelectionError::NoSurfaceSelected)
    );

    assert!(panel.select_surface(73));
    assert_eq!(
        panel.bake_selected_action(),
        Ok(NavigationBakeAction::bake_selected_surface(73, false))
    );
}

#[test]
fn navigation_bake_commands_keep_operation_payload_and_undo_contracts() {
    let registration = plugin_registration();
    for (operation, payload_schema) in [
        (NAVIGATION_BAKE_SCENE_OPERATION, "navigation.bake.scene.v1"),
        (
            NAVIGATION_BAKE_SURFACE_OPERATION,
            "navigation.bake.selected_surface.v1",
        ),
        (
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            "navigation.bake.clear_surface.v1",
        ),
    ] {
        let operation =
            zircon_editor::core::editor_operation::EditorOperationPath::parse(operation).unwrap();
        let command = registration
            .extensions
            .pending_command(&operation)
            .expect("navigation bake command must remain an edit operation");
        assert!(command.event().is_none());
        assert_eq!(command.payload_schema_id(), Some(payload_schema));
        assert!(matches!(
            command.action(),
            zircon_editor::core::commands::EditorCommandAction::Operation
        ));
        assert_eq!(
            registration
                .extensions
                .operation_factory(&operation)
                .map(|factory| factory.undo_display_name()),
            Some(command.display_name())
        );
    }
}

#[test]
fn navigation_overlay_command_does_not_impersonate_a_scene_mode() {
    let registration = plugin_registration();
    assert!(registration.extensions.scene_mode_descriptors().is_empty());
    assert!(registration.extensions.menu_items().iter().any(|item| {
        item.path() == "View/Debug Overlays/Navigation"
            && item.operation().as_str() == NAVIGATION_TOGGLE_GIZMOS_OPERATION
    }));
}

#[test]
fn navigation_overlay_controller_toggles_and_submits_to_viewport_sink() {
    let snapshot = NavigationGizmoSnapshot {
        triangles: vec![NavigationGizmoTriangle {
            vertices: [[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
            area: AREA_WALKABLE,
            tile: 0,
        }],
        off_mesh_links: Vec::new(),
    };
    let mut controller = NavigationOverlayController::new(RecordingOverlaySink::default());

    assert!(!controller.publish(1, &snapshot, None));
    assert!(controller.toggle());
    assert!(controller.publish(1, &snapshot, None));
    assert!(controller.sink().submissions.last().unwrap().is_some());
    assert!(!controller.toggle());
    assert!(controller.sink().submissions.last().unwrap().is_none());

    let operation = zircon_editor::core::editor_operation::EditorOperationPath::parse(
        NAVIGATION_TOGGLE_GIZMOS_OPERATION,
    )
    .unwrap();
    let registration = plugin_registration();
    let command = registration.extensions.pending_command(&operation).unwrap();
    assert_eq!(
        command.payload_schema_id(),
        Some("navigation.overlay.toggle.v1")
    );
    assert!(command.event().is_none());
}

#[test]
fn navigation_overlay_contains_area_mesh_agent_path_and_avoidance_vectors() {
    let nav_mesh = NavigationGizmoSnapshot {
        triangles: vec![
            NavigationGizmoTriangle {
                vertices: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                area: AREA_WALKABLE,
                tile: 0,
            },
            NavigationGizmoTriangle {
                vertices: [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
                area: AREA_JUMP,
                tile: 0,
            },
        ],
        off_mesh_links: Vec::new(),
    };
    let frame = NavigationPieFrame::new(
        7,
        1,
        NavigationOverlayFrame {
            owner_generation: 1,
            nav_mesh: nav_mesh.clone(),
            tick_report: NavAgentTickReport {
                debug_agents: vec![NavigationAgentDebugState {
                    entity: 9,
                    position: [0.2, 0.0, 0.2],
                    destination: Some([1.8, 0.0, 1.8]),
                    desired_velocity: [1.0, 0.0, 0.0],
                    avoidance_velocity: [0.0, 0.0, 0.5],
                    path_status: Some(NavPathStatus::Complete),
                    path: vec![[0.2, 0.0, 0.2], [1.0, 0.0, 1.0], [1.8, 0.0, 1.8]],
                }],
                ..NavAgentTickReport::default()
            },
        },
    );
    let overlay = build_navigation_overlay(
        100,
        &nav_mesh,
        Some(&frame),
        NavigationOverlayOptions::default(),
    );

    assert_eq!(overlay.kind, SceneGizmoKind::NavigationMesh);
    assert!(overlay.lines.len() >= 9, "mesh edges + path + two vectors");
    assert_ne!(overlay.lines[0].color, overlay.lines[3].color);
    assert!(!overlay.pick_shapes.is_empty());
}

#[test]
fn navigation_pie_mirror_rejects_cross_session_and_out_of_order_frames() {
    let mut mirror = NavigationPieMirror::default();
    mirror.begin_session(12);
    let agent = NavigationAgentDebugState {
        entity: 44,
        position: [1.0, 0.0, 2.0],
        destination: Some([5.0, 0.0, 8.0]),
        desired_velocity: [0.5, 0.0, 0.25],
        avoidance_velocity: [0.0, 0.0, 0.1],
        path_status: Some(NavPathStatus::Partial),
        path: vec![[1.0, 0.0, 2.0], [3.0, 0.0, 4.0]],
    };
    assert_eq!(
        mirror.apply_overlay_frame(
            12,
            2,
            NavigationOverlayFrame {
                owner_generation: 4,
                nav_mesh: NavigationGizmoSnapshot::default(),
                tick_report: NavAgentTickReport {
                    debug_agents: vec![agent.clone()],
                    ..NavAgentTickReport::default()
                },
            },
        ),
        NavigationPieMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_overlay_frame(12, 1, NavigationOverlayFrame::default()),
        NavigationPieMirrorApply::Stale
    );
    assert_eq!(
        mirror.apply_overlay_frame(99, 3, NavigationOverlayFrame::default()),
        NavigationPieMirrorApply::WrongSession
    );
    assert_eq!(
        mirror.apply_overlay_frame(
            12,
            3,
            NavigationOverlayFrame {
                owner_generation: 3,
                ..NavigationOverlayFrame::default()
            },
        ),
        NavigationPieMirrorApply::StaleOwnerGeneration
    );
    assert_eq!(mirror.agent(44), Some(&agent));
    assert_eq!(mirror.sequence(), Some(2));
}

#[test]
fn navigation_m6_layout_exposes_bake_progress_and_pie_debug_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bake = std::fs::read_to_string(root.join("bake.zui")).unwrap();
    for marker in [
        "NavigationBakeSceneButton",
        "NavigationBakeSelectedButton",
        "NavigationClearBakeButton",
        "NavigationBakeProgress",
        "NavigationBakeDiagnostics",
    ] {
        assert!(bake.contains(marker), "bake layout missing {marker}");
    }

    let debug = std::fs::read_to_string(root.join("debug_gizmos.zui")).unwrap();
    for marker in [
        "NavigationAreaOverlayToggle",
        "NavigationAgentPathToggle",
        "NavigationAvoidanceVectorToggle",
        "NavigationPieMirrorStatus",
        "NavigationDebugAgentList",
    ] {
        assert!(debug.contains(marker), "debug layout missing {marker}");
    }
}
