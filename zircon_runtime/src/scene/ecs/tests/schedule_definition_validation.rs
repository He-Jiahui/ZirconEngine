use super::*;
use crate::scene::ecs::{SceneScheduleStagePlan, Schedule};

fn descriptor(id: &str, order: i32) -> SceneSystemDescriptor {
    SceneSystemDescriptor::new(id, SystemStage::Update, InternalSceneSystem::WorldTransform)
        .with_order(order)
}

#[test]
fn persisted_registry_rejects_duplicate_and_invalid_system_ids() {
    let duplicate = serde_json::to_value([descriptor("same", 0), descriptor("same", 1)]).unwrap();
    let error = serde_json::from_value::<SceneSystemRegistry>(duplicate).unwrap_err();
    assert!(error.to_string().contains("already registered"));
    for id in ["", " ", " surrounded "] {
        let invalid = serde_json::to_value([descriptor(id, 0)]).unwrap();
        assert!(serde_json::from_value::<SceneSystemRegistry>(invalid).is_err());
    }
}

#[test]
fn persisted_registry_uses_registration_order_and_round_trips() {
    let document = serde_json::to_value([
        descriptor("later", 7),
        descriptor("b", 2),
        descriptor("a", 2),
    ])
    .unwrap();
    let registry: SceneSystemRegistry = serde_json::from_value(document).unwrap();
    let ids = registry
        .systems()
        .iter()
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["a", "b", "later"]);
    let decoded: SceneSystemRegistry =
        serde_json::from_value(serde_json::to_value(&registry).unwrap()).unwrap();
    assert_eq!(decoded.systems(), registry.systems());
}

#[test]
fn persisted_schedule_rejects_repeated_stages_before_compiling_buckets() {
    let document = serde_json::json!({ "stages": ["Update", "Update"], "systems": [] });
    let error = serde_json::from_value::<Schedule>(document).unwrap_err();
    assert!(error.to_string().contains("appears more than once"));
    assert_eq!(
        SceneScheduleStagePlan::from_registry(
            &[SystemStage::Update, SystemStage::Update],
            &SceneSystemRegistry::new(),
        )
        .unwrap_err(),
        ScheduleError::DuplicateStage(SystemStage::Update),
    );
}
