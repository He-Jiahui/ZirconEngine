use crate::{
    package_manifest, AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID,
    AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
};
use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiBlackboardEntry,
    AiBlackboardValue, AiDecisionStatus, AiPerceptionDebugSnapshot, AiPerceptionSense,
    AiPerceptionSnapshot, AiPerceptionStimulus, BtNodeResultEvent,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

#[test]
fn behavior_debug_snapshot_is_declared_as_a_typed_runtime_event() {
    let manifest = package_manifest();
    let event = manifest
        .event_catalogs
        .iter()
        .flat_map(|catalog| &catalog.events)
        .find(|event| event.id == AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID)
        .expect("AI behavior debug event manifest");
    assert_eq!(
        event.payload_schema,
        AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA
    );
    let node_event = manifest
        .event_catalogs
        .iter()
        .flat_map(|catalog| &catalog.events)
        .find(|event| event.id == crate::BT_NODE_RESULT_EVENT_ID)
        .expect("behavior-tree node-result event manifest");
    assert_eq!(
        node_event.payload_schema,
        crate::BT_NODE_RESULT_PAYLOAD_SCHEMA
    );

    let frame = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(7),
            entity: 19,
            status: AiDecisionStatus::Running,
            active_node: Some("move_to".to_owned()),
            diagnostic: None,
        },
        behavior_tree: Some("patrol".to_owned()),
        blackboard: vec![AiBlackboardEntry::new(
            "target",
            AiBlackboardValue::Entity(44),
        )],
        perception: Some(AiPerceptionSnapshot {
            agent: 19,
            stimuli: vec![AiPerceptionStimulus {
                source: 44,
                sense: AiPerceptionSense::Sight,
                position: Vec3::new(2.0, 0.0, 3.0),
                strength: 0.75,
                age_seconds: 0.2,
            }],
        }),
        perception_debug: Some(AiPerceptionDebugSnapshot {
            position: Vec3::new(1.0, 0.0, 2.0),
            forward: Vec3::Z,
            sight_fov_degrees: 90.0,
            sight_range: 12.0,
            hearing_radius: 8.0,
        }),
    };
    let snapshot = AiBehaviorDebugSnapshot {
        world: WorldHandle::new(7),
        frames: vec![frame],
    };
    assert_eq!(snapshot.frames.len(), 1);
    assert_eq!(snapshot.frames[0].report.entity, 19);
    assert_eq!(snapshot.frames[0].behavior_tree.as_deref(), Some("patrol"));
    assert_eq!(snapshot.frames[0].blackboard.len(), 1);
    assert_eq!(
        snapshot.frames[0]
            .perception
            .as_ref()
            .expect("perception snapshot")
            .stimuli
            .len(),
        1
    );
    assert_eq!(
        snapshot.frames[0]
            .perception_debug
            .as_ref()
            .expect("perception debug snapshot")
            .sight_range,
        12.0
    );
    assert_eq!(
        snapshot.frames[0].report.node_result_event(),
        Some(BtNodeResultEvent {
            world: WorldHandle::new(7),
            entity: 19,
            node_id: "move_to".to_owned(),
            status: AiDecisionStatus::Running,
            diagnostic: None,
        })
    );
}
