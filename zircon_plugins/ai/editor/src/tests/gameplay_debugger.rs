use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiBlackboardEntry,
    AiBlackboardValue,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use crate::{AiPieMirror, AiPieMirrorApply};

fn frame(world: u64, entity: u64, value: i64) -> AiBehaviorDebugFrame {
    AiBehaviorDebugFrame {
        report: AiAgentTickReport::idle(WorldHandle::new(world), entity),
        behavior_tree: Some("patrol".to_owned()),
        blackboard: vec![AiBlackboardEntry::new(
            "score",
            AiBlackboardValue::Integer(value),
        )],
        perception: None,
        perception_debug: None,
    }
}

fn snapshot(world: u64, frames: Vec<AiBehaviorDebugFrame>) -> AiBehaviorDebugSnapshot {
    AiBehaviorDebugSnapshot {
        world: WorldHandle::new(world),
        frames,
    }
}

#[test]
fn another_world_delivery_does_not_relabel_the_selected_agents_frame() {
    let mut mirror = AiPieMirror::default();
    mirror.begin_session(12);
    for (world, sequence, value) in [(7, 1, 10), (8, 2, 20), (7, 3, 30)] {
        assert_eq!(
            mirror.apply_debug_snapshot(
                12,
                sequence,
                snapshot(world, vec![frame(world, 19, value)])
            ),
            AiPieMirrorApply::Applied
        );
    }
    let selected = mirror
        .agent_frame_in_world(&WorldHandle::new(8), 19)
        .unwrap();
    assert_eq!(selected.play_session_id(), 12);
    assert_eq!(selected.delivery_sequence(), 2);
    assert_eq!(selected.frame(), &frame(8, 19, 20));
    assert_eq!(mirror.sequence(), Some(3));
    assert!(std::ptr::eq(
        selected.frame(),
        mirror.agent_in_world(&WorldHandle::new(8), 19).unwrap()
    ));
    assert!(mirror.agent(19).is_none());
    assert_eq!(mirror.agents().len(), 2);
    assert_eq!(mirror.agents_in_world(&WorldHandle::new(7)).count(), 1);
}

#[test]
fn rejected_deliveries_leave_payload_and_original_provenance_together() {
    let mut mirror = AiPieMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_debug_snapshot(12, 4, snapshot(7, vec![frame(7, 19, 10)])),
        AiPieMirrorApply::Applied
    );
    for (session, sequence, payload, expected) in [
        (
            12,
            3,
            snapshot(7, vec![frame(7, 19, 30)]),
            AiPieMirrorApply::Stale,
        ),
        (
            99,
            5,
            snapshot(7, vec![frame(7, 19, 40)]),
            AiPieMirrorApply::WrongSession,
        ),
        (
            12,
            5,
            snapshot(7, vec![frame(8, 19, 50)]),
            AiPieMirrorApply::InvalidSnapshotWorld,
        ),
    ] {
        assert_eq!(
            mirror.apply_debug_snapshot(session, sequence, payload),
            expected
        );
        let selected = mirror
            .agent_frame_in_world(&WorldHandle::new(7), 19)
            .unwrap();
        assert_eq!(selected.play_session_id(), 12);
        assert_eq!(selected.delivery_sequence(), 4);
        assert_eq!(selected.frame(), &frame(7, 19, 10));
        assert_eq!(mirror.sequence(), Some(4));
    }
}

#[test]
fn an_empty_world_snapshot_removes_only_that_worlds_agent_frame() {
    let mut mirror = AiPieMirror::default();
    mirror.begin_session(12);
    for (world, sequence) in [(7, 1), (8, 2)] {
        assert_eq!(
            mirror.apply_debug_snapshot(12, sequence, snapshot(world, vec![frame(world, 19, 10)])),
            AiPieMirrorApply::Applied
        );
    }
    assert_eq!(
        mirror.apply_debug_snapshot(12, 3, snapshot(7, Vec::new())),
        AiPieMirrorApply::Applied
    );
    assert!(mirror
        .agent_frame_in_world(&WorldHandle::new(7), 19)
        .is_none());
    let retained = mirror
        .agent_frame_in_world(&WorldHandle::new(8), 19)
        .unwrap();
    assert_eq!(retained.delivery_sequence(), 2);
    assert_eq!(retained.frame(), &frame(8, 19, 10));
    assert_eq!(mirror.agents().len(), 1);
}

#[test]
fn ending_and_restarting_a_session_cannot_reuse_old_frame_provenance() {
    let mut mirror = AiPieMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_debug_snapshot(12, 4, snapshot(7, vec![frame(7, 19, 10)])),
        AiPieMirrorApply::Applied
    );
    assert!(!mirror.end_session(99));
    assert!(mirror
        .agent_frame_in_world(&WorldHandle::new(7), 19)
        .is_some());
    assert!(mirror.end_session(12));
    assert!(mirror
        .agent_frame_in_world(&WorldHandle::new(7), 19)
        .is_none());
    assert_eq!(mirror.play_session_id(), None);
    assert_eq!(mirror.sequence(), None);
    mirror.begin_session(13);
    assert!(mirror
        .agent_frame_in_world(&WorldHandle::new(7), 19)
        .is_none());
    assert_eq!(
        mirror.apply_debug_snapshot(12, 5, snapshot(7, vec![frame(7, 19, 20)])),
        AiPieMirrorApply::WrongSession
    );
    assert_eq!(
        mirror.apply_debug_snapshot(13, 1, snapshot(7, vec![frame(7, 19, 30)])),
        AiPieMirrorApply::Applied
    );
    let selected = mirror
        .agent_frame_in_world(&WorldHandle::new(7), 19)
        .unwrap();
    assert_eq!(selected.play_session_id(), 13);
    assert_eq!(selected.delivery_sequence(), 1);
    assert_eq!(selected.frame(), &frame(7, 19, 30));
}
