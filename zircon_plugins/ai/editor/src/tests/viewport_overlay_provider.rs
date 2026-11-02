use std::sync::Arc;

use zircon_editor::EditorPlugin;
use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiDecisionStatus,
    AiPerceptionDebugSnapshot,
};
use zircon_runtime::core::framework::render::SceneGizmoKind;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::extension_ids::AI_BEHAVIOR_DEBUG_CONSUMER_ID;
use crate::viewport_overlay_provider::AiViewportOverlayProvider;
use crate::{AiPieMirror, AiPieMirrorApply};

#[test]
fn cached_ai_metadata_keeps_registration_values_and_runtime_mirrors_independent() {
    let first = crate::editor_plugin();
    let expected_manifest = first.package_manifest();
    assert!(!expected_manifest.components.is_empty());
    first.pie_mirror().lock().unwrap().begin_session(17);
    let mut report = first.registration_report();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    report.package_manifest.components.clear();
    report.capabilities.clear();
    report
        .diagnostics
        .push("caller-local diagnostic".to_owned());

    let second = crate::editor_plugin();
    assert!(!Arc::ptr_eq(&first.pie_mirror(), &second.pie_mirror()));
    assert!(!Arc::ptr_eq(
        &first.node_result_mirror(),
        &second.node_result_mirror()
    ));
    assert_eq!(second.pie_mirror().lock().unwrap().play_session_id(), None);
    let fresh = second.registration_report();
    assert!(fresh.is_success(), "{:?}", fresh.diagnostics);
    assert_eq!(fresh.package_manifest, expected_manifest);
    assert_eq!(fresh.capabilities, crate::editor_capabilities());
    assert_eq!(
        first.pie_mirror().lock().unwrap().play_session_id(),
        Some(17)
    );
}

#[test]
fn ai_provider_reads_live_declared_mirror_and_clears_replaced_worlds_and_sessions() {
    let plugin = crate::editor_plugin();
    let mirror = plugin.pie_mirror();
    let consumers = plugin.runtime_event_consumers();
    let declared_mirror = consumers
        .registration(AI_BEHAVIOR_DEBUG_CONSUMER_ID)
        .unwrap()
        .state::<AiPieMirror>()
        .unwrap();
    assert!(Arc::ptr_eq(&mirror, &declared_mirror));
    assert!(!Arc::ptr_eq(&mirror, &crate::editor_plugin().pie_mirror()));

    let provider = AiViewportOverlayProvider::new(mirror.clone());
    assert!(provider.extract_current(Some(88)).is_empty());
    {
        let mut state = declared_mirror.lock().unwrap();
        state.begin_session(17);
        for (sequence, world, position) in [
            (1, 3, Vec3::new(1.0, 0.0, 2.0)),
            (2, 4, Vec3::new(30.0, 0.0, 40.0)),
        ] {
            assert_eq!(
                state.apply_debug_snapshot(17, sequence, snapshot(world, position)),
                AiPieMirrorApply::Applied
            );
        }
        assert_eq!(
            state.apply_debug_snapshot(99, 3, snapshot(3, Vec3::ZERO)),
            AiPieMirrorApply::WrongSession
        );
        assert_eq!(
            state.apply_debug_snapshot(17, 2, snapshot(3, Vec3::ZERO)),
            AiPieMirrorApply::Stale
        );
    }
    let overlays = provider.extract_current(Some(88));
    assert_eq!(overlays.len(), 2);
    for (overlay, position) in overlays
        .iter()
        .zip([Vec3::new(1.0, 0.0, 2.0), Vec3::new(30.0, 0.0, 40.0)])
    {
        assert_eq!(overlay.owner, 88);
        assert_eq!(overlay.kind, SceneGizmoKind::AiPerception);
        assert!(!overlay.lines.is_empty());
        assert_eq!(overlay.pick_shapes.len(), 2);
        assert!(matches!(
            &overlay.pick_shapes[0],
            zircon_runtime::core::framework::render::OverlayPickShape::Sphere { center, .. }
                if *center == position
        ));
    }

    assert_eq!(
        mirror.lock().unwrap().apply_debug_snapshot(
            17,
            3,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(3),
                frames: Vec::new(),
            },
        ),
        AiPieMirrorApply::Applied
    );
    assert_eq!(provider.extract_current(None).len(), 1);
    assert!(!mirror.lock().unwrap().end_session(99));
    assert_eq!(provider.extract_current(None).len(), 1);
    assert!(mirror.lock().unwrap().end_session(17));
    assert!(provider.extract_current(Some(88)).is_empty());

    mirror.lock().unwrap().begin_session(18);
    assert!(provider.extract_current(None).is_empty());
}

fn snapshot(world: u64, position: Vec3) -> AiBehaviorDebugSnapshot {
    AiBehaviorDebugSnapshot {
        world: WorldHandle::new(world),
        frames: vec![AiBehaviorDebugFrame {
            report: AiAgentTickReport {
                world: WorldHandle::new(world),
                entity: 27,
                status: AiDecisionStatus::Running,
                active_node: None,
                diagnostic: None,
            },
            behavior_tree: None,
            blackboard: Vec::new(),
            perception: None,
            perception_debug: Some(AiPerceptionDebugSnapshot {
                position,
                forward: Vec3::Z,
                sight_fov_degrees: 90.0,
                sight_range: 6.0,
                hearing_radius: 4.0,
            }),
        }],
    }
}
