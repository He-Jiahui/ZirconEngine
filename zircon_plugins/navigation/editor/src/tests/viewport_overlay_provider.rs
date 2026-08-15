use std::sync::{Arc, Mutex};

use zircon_plugin_navigation_runtime::NavigationOverlayFrame;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavPathStatus, NavigationAgentDebugState, NavigationGizmoSnapshot,
    NavigationGizmoTriangle, AREA_WALKABLE,
};
use zircon_runtime::core::framework::render::SceneGizmoKind;

use crate::runtime_mirror::NavigationPieMirror;
use crate::viewport_overlay_provider::{
    navigation_viewport_overlay_provider_registration, NavigationViewportOverlayProvider,
};
use crate::{NAVIGATION_GIZMOS_CAPABILITY, NAVIGATION_OVERLAY_PROVIDER_ID};

#[test]
fn registered_navigation_provider_extracts_shared_frame_and_clears_after_pie() {
    let mirror = Arc::new(Mutex::new(NavigationPieMirror::default()));
    {
        let mut state = mirror.lock().unwrap();
        state.begin_session(17);
        assert_eq!(
            state.apply_overlay_frame(17, 1, overlay_frame()),
            crate::NavigationPieMirrorApply::Applied
        );
    }

    let registration = navigation_viewport_overlay_provider_registration(mirror.clone());
    assert_eq!(registration.provider_id(), NAVIGATION_OVERLAY_PROVIDER_ID);
    assert_eq!(
        registration.required_capabilities(),
        &[NAVIGATION_GIZMOS_CAPABILITY.to_string()]
    );

    let provider = NavigationViewportOverlayProvider::new(mirror.clone());
    let extracts = provider.extract_current(Some(88));
    assert_eq!(extracts.len(), 1);
    let overlay = &extracts[0];
    assert_eq!(overlay.owner, 88);
    assert_eq!(overlay.kind, SceneGizmoKind::NavigationMesh);
    assert!(
        overlay.lines.len() >= 6,
        "mesh, path, and vectors are present"
    );
    assert!(!overlay.pick_shapes.is_empty());

    assert!(mirror.lock().unwrap().end_session(17));
    assert!(provider.extract_current(Some(88)).is_empty());
}

fn overlay_frame() -> NavigationOverlayFrame {
    NavigationOverlayFrame {
        owner_generation: 9,
        nav_mesh: NavigationGizmoSnapshot {
            triangles: vec![NavigationGizmoTriangle {
                vertices: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                area: AREA_WALKABLE,
                tile: 0,
            }],
            off_mesh_links: Vec::new(),
        },
        tick_report: NavAgentTickReport {
            debug_agents: vec![NavigationAgentDebugState {
                entity: 5,
                position: [0.0, 0.0, 0.0],
                destination: Some([2.0, 0.0, 2.0]),
                desired_velocity: [1.0, 0.0, 0.0],
                avoidance_velocity: [0.0, 0.0, 0.5],
                path_status: Some(NavPathStatus::Complete),
                path: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 2.0]],
            }],
            ..NavAgentTickReport::default()
        },
    }
}
