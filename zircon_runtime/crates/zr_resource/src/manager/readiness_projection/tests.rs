use super::*;
use crate::{ResourceKind, ResourceLocator};

mod behavior_red;
mod profile;

fn source_update(record: ResourceRecord) -> ResourceReadinessSourceUpdate {
    ResourceReadinessSourceUpdate {
        id: record.id,
        record: Some(record),
        runtime_state: RuntimeResourceState::Loaded,
        payload_type_id: Some(TypeId::of::<()>()),
    }
}

fn ready_record(label: &str, dependency_ids: Vec<ResourceId>) -> ResourceRecord {
    let locator = ResourceLocator::parse(&format!("res://readiness/{label}.asset"))
        .expect("valid readiness locator");
    ResourceRecord::new(
        ResourceId::from_locator(&locator),
        ResourceKind::Data,
        locator,
    )
    .with_state(ResourceState::Ready)
    .with_dependency_ids(dependency_ids)
}

#[test]
fn identical_updates_preserve_source_and_generation_identity() {
    let locator = ResourceLocator::parse("res://models/readiness-noop.glb")
        .expect("valid readiness no-op locator");
    let id = ResourceId::from_locator(&locator);
    let record =
        ResourceRecord::new(id, ResourceKind::Model, locator).with_state(ResourceState::Ready);
    let mut projection = ResourceReadinessProjection::default();
    projection.apply_updates([source_update(record.clone())]);
    let generation = projection.generation();
    let source_record = projection
        .sources
        .get(&id)
        .expect("readiness source")
        .record
        .clone();

    projection.apply_updates([source_update(record)]);

    let repeated_generation = projection.generation();
    assert!(Arc::ptr_eq(&generation, &repeated_generation));
    assert!(Arc::ptr_eq(
        &source_record,
        &projection
            .sources
            .get(&id)
            .expect("unchanged readiness source")
            .record
    ));

    let mut empty_projection = ResourceReadinessProjection::default();
    let empty_generation = empty_projection.generation();
    empty_projection.apply_updates([ResourceReadinessSourceUpdate {
        id,
        record: None,
        runtime_state: RuntimeResourceState::Loaded,
        payload_type_id: Some(TypeId::of::<()>()),
    }]);
    let repeated_empty_generation = empty_projection.generation();

    assert!(Arc::ptr_eq(&empty_generation, &repeated_empty_generation));
    assert!(empty_projection.sources.is_empty());
}

#[test]
fn removing_the_last_reverse_edge_reclaims_its_dependency_bucket() {
    let dependency_id = ResourceId::from_stable_label("temporary-readiness-dependency");
    let parent_locator =
        ResourceLocator::parse("res://models/readiness-parent.glb").expect("valid parent locator");
    let parent_id = ResourceId::from_locator(&parent_locator);
    let parent = ResourceRecord::new(parent_id, ResourceKind::Model, parent_locator)
        .with_state(ResourceState::Ready)
        .with_dependency_ids(vec![dependency_id]);
    let mut projection = ResourceReadinessProjection::default();

    projection.apply_updates([source_update(parent.clone())]);
    assert_eq!(projection.reverse_dependencies.len(), 1);

    projection.apply_updates([source_update(parent.with_dependency_ids(Vec::new()))]);

    assert!(projection.reverse_dependencies.is_empty());
}
