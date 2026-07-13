use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
};

use crate::blackboard::{BlackboardLayout, BlackboardLayoutError, BlackboardStore};

fn schema() -> AiBlackboardSchemaDescriptor {
    AiBlackboardSchemaDescriptor::new("combat", "Combat")
        .with_key("alert", "bool", true)
        .with_key("target", "entity", false)
        .with_key("distance", "scalar", false)
}

#[test]
fn schema_layout_round_trip() {
    let layout = Arc::new(BlackboardLayout::from_schema(&schema()).expect("valid layout"));
    let alert = layout.resolve("alert").expect("alert slot");
    let target = layout.resolve("target").expect("target slot");
    assert_eq!(alert.offset(), 0);
    assert_eq!(
        target.offset(),
        0,
        "type partitions use independent offsets"
    );
    assert_ne!(alert.generation_index(), target.generation_index());

    let mut store = BlackboardStore::new(layout);
    store
        .write("alert", AiBlackboardValue::Bool(true))
        .expect("write alert");
    store
        .write("target", AiBlackboardValue::Entity(42))
        .expect("write target");
    assert_eq!(
        store.entries(),
        vec![
            AiBlackboardEntry::new("alert", AiBlackboardValue::Bool(true)),
            AiBlackboardEntry::new("target", AiBlackboardValue::Entity(42)),
        ]
    );
}

#[test]
fn write_same_value_does_not_bump_generation() {
    let layout = Arc::new(BlackboardLayout::from_schema(&schema()).expect("valid layout"));
    let slot = layout.resolve("alert").expect("alert slot");
    let mut store = BlackboardStore::new(layout);
    let first = store
        .write("alert", AiBlackboardValue::Bool(true))
        .expect("first write");
    let same = store
        .write("alert", AiBlackboardValue::Bool(true))
        .expect("same write");
    let changed = store
        .write("alert", AiBlackboardValue::Bool(false))
        .expect("changed write");

    assert!(first.changed);
    assert_eq!(first.generation, 1);
    assert!(!same.changed);
    assert_eq!(same.generation, 1);
    assert!(changed.changed);
    assert_eq!(store.generation(slot), 2);
}

#[test]
fn synchronize_failure_is_atomic() {
    let layout = Arc::new(BlackboardLayout::from_schema(&schema()).expect("valid layout"));
    let alert = layout.resolve("alert").expect("alert slot");
    let mut store = BlackboardStore::new(layout);
    store
        .write("alert", AiBlackboardValue::Bool(false))
        .expect("initial write");
    store.drain_changed_slots();
    let entries_before = store.entries();
    let generation_before = store.generation(alert);

    store
        .synchronize(&[
            AiBlackboardEntry::new("alert", AiBlackboardValue::Bool(true)),
            AiBlackboardEntry::new("target", AiBlackboardValue::Bool(true)),
        ])
        .expect_err("second entry has the wrong type");

    assert_eq!(store.entries(), entries_before);
    assert_eq!(store.generation(alert), generation_before);
    assert!(store.drain_changed_slots().is_empty());
}

#[test]
fn layout_rejects_invalid_types_and_duplicate_keys() {
    let invalid_type = AiBlackboardSchemaDescriptor::new("invalid", "Invalid").with_key(
        "value",
        "quaternion",
        false,
    );
    assert!(matches!(
        BlackboardLayout::from_schema(&invalid_type),
        Err(BlackboardLayoutError::UnknownValueType { .. })
    ));

    let duplicate = AiBlackboardSchemaDescriptor::new("duplicate", "Duplicate")
        .with_key("value", "bool", false)
        .with_key("value", "integer", false);
    assert!(matches!(
        BlackboardLayout::from_schema(&duplicate),
        Err(BlackboardLayoutError::DuplicateKey { .. })
    ));
}
