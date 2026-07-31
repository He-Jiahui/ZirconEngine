use std::collections::BTreeMap;

use serde_json::json;

use crate::core::framework::scene::{
    ComponentPropertyPath, ComponentTypeDescriptor, ScenePropertyValue,
};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{MeshRenderer, Name};
use crate::scene::{EntityId, NodeKind, World, WorldInspection};

fn hashes_by_entity(world: &World) -> BTreeMap<EntityId, u64> {
    world
        .inspect_hierarchy()
        .into_iter()
        .map(|row| (row.entity, row.subtree_hash))
        .collect()
}

fn world_with_serialized_parents(world: &World, parents: &[(EntityId, Option<EntityId>)]) -> World {
    let mut encoded = serde_json::to_value(world).unwrap();
    let hierarchy = encoded
        .get_mut("hierarchy")
        .and_then(serde_json::Value::as_object_mut)
        .expect("serialized world hierarchy should be an object");
    for (entity, parent) in parents {
        let row = hierarchy
            .get_mut(&entity.to_string())
            .and_then(serde_json::Value::as_object_mut)
            .expect("serialized hierarchy row should be an object");
        row.insert("parent".to_string(), serde_json::to_value(parent).unwrap());
    }
    serde_json::from_value(encoded).unwrap()
}

#[test]
fn split_inspection_entries_compose_to_the_world_snapshot() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);

    let mut hierarchy_rows = world.inspect_hierarchy();
    for row in &mut hierarchy_rows {
        row.focused = row.entity == entity;
    }
    let fields = world.inspect_fields(entity);

    assert_eq!(
        WorldInspection::from_world(&world, Some(entity)),
        WorldInspection {
            generation: world.world_generation(),
            focused_entity: Some(entity),
            hierarchy_rows,
            fields,
        }
    );
    assert!(world.inspect_fields(entity + 1).is_empty());
}

#[test]
fn inspection_artifact_reuses_its_arc_until_the_world_generation_changes() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);

    let first = world.inspection_artifact();
    let initial_diagnostics = world.inspection_artifact_diagnostics();
    let stable = world.inspection_artifact();
    assert!(std::sync::Arc::ptr_eq(&first, &stable));
    assert_eq!(first.generation(), world.world_generation());
    assert_eq!(world.inspection_artifact_diagnostics(), initial_diagnostics);

    world.rename_node(entity, "Renamed").unwrap();
    let changed = world.inspection_artifact();
    assert!(!std::sync::Arc::ptr_eq(&first, &changed));
    assert_eq!(changed.generation(), world.world_generation());
    assert_eq!(
        world.inspection_artifact_diagnostics().hierarchy_builds(),
        initial_diagnostics.hierarchy_builds() + 1
    );
}

#[test]
fn inspection_artifact_looks_up_hierarchy_rows_by_stable_entity_identity() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Cube);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let artifact = world.inspection_artifact();

    assert_eq!(
        artifact.hierarchy_row(parent).map(|row| row.entity),
        Some(parent)
    );
    assert_eq!(
        artifact.hierarchy_row(child).map(|row| row.parent),
        Some(Some(parent))
    );
    assert_eq!(artifact.hierarchy_row(child + 100), None);
}

#[test]
fn summary_component_change_rebuilds_the_hierarchy_artifact() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    let initial = world.inspection_artifact();
    assert_eq!(initial.summary().mesh_count(), 0);

    world.insert(entity, MeshRenderer::default()).unwrap();
    let current = world.inspection_artifact();

    assert_eq!(current.summary().mesh_count(), 1);
    assert_eq!(
        world.inspection_artifact_diagnostics().hierarchy_builds(),
        2
    );
}

#[test]
fn property_edit_publishes_a_generation_without_rebuilding_unchanged_hierarchy_rows() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Cube);
    let initial = world.inspection_artifact();
    let initial_fields = world
        .inspection_fields_artifact(entity)
        .expect("cube should expose reflected fields");
    let initial_diagnostics = world.inspection_artifact_diagnostics();
    let render_queue = ComponentPropertyPath::parse("MeshRenderer.render_queue").unwrap();

    assert!(
        world
            .set_property(entity, &render_queue, ScenePropertyValue::Integer(1))
            .unwrap()
    );
    let current = world.inspection_artifact();
    let current_fields = world
        .inspection_fields_artifact(entity)
        .expect("edited cube should expose reflected fields");
    let hierarchy_delta = current
        .published_delta_from(initial.generation())
        .expect("the adjacent generation should carry an inspection delta");
    let field_delta = current_fields.delta_from(&initial_fields);

    assert_eq!(current.generation(), initial.generation() + 1);
    assert!(hierarchy_delta.added_rows().is_empty());
    assert!(hierarchy_delta.changed_rows().is_empty());
    assert!(hierarchy_delta.removed_entities().is_empty());
    assert!(field_delta.changed_fields().iter().any(|field| {
        field.component_type_path.ends_with("::MeshRenderer") && field.field_name == "render_queue"
    }));
    assert_eq!(
        world.inspection_artifact_diagnostics().hierarchy_builds(),
        initial_diagnostics.hierarchy_builds()
    );
}

#[test]
fn direct_dynamic_component_mutations_publish_fresh_field_artifacts() {
    const DYNAMIC_TYPE_PATH: &str = "weather.Component.InspectionCache";

    let mut world = World::empty();
    world
        .register_component_type(
            ComponentTypeDescriptor::new(DYNAMIC_TYPE_PATH, "weather", "Inspection Cache")
                .with_property("coverage", "Scalar", true),
        )
        .expect("dynamic descriptor should register");
    let entity = world.spawn_node(NodeKind::Empty);
    let initial = world.inspection_artifact();
    let initial_fields = world
        .inspection_fields_artifact(entity)
        .expect("empty node should expose reflected fields");
    let initial_diagnostics = world.inspection_artifact_diagnostics();

    assert!(world
        .set_dynamic_component(entity, DYNAMIC_TYPE_PATH, json!({ "coverage": 0.75 }))
        .expect("direct dynamic component insertion should succeed"));
    let added = world.inspection_artifact();
    let added_fields = world
        .inspection_fields_artifact(entity)
        .expect("added dynamic component should be reflected");

    assert_eq!(added.generation(), initial.generation() + 1);
    assert_eq!(
        world.inspection_artifact_diagnostics().hierarchy_builds(),
        initial_diagnostics.hierarchy_builds()
    );
    assert!(added
        .published_delta_from(initial.generation())
        .expect("the adjacent generation should publish a delta")
        .changed_rows()
        .is_empty());
    assert!(added_fields
        .delta_from(&initial_fields)
        .changed_fields()
        .iter()
        .any(|field| field.component_type_path == DYNAMIC_TYPE_PATH
            && field.field_name == "coverage"));

    assert!(world
        .remove_dynamic_component(entity, DYNAMIC_TYPE_PATH)
        .expect("direct dynamic component removal should succeed"));
    let removed = world.inspection_artifact();
    let removed_fields = world
        .inspection_fields_artifact(entity)
        .expect("remaining fixed components should still be reflected");

    assert_eq!(removed.generation(), added.generation() + 1);
    assert_eq!(
        world.inspection_artifact_diagnostics().hierarchy_builds(),
        initial_diagnostics.hierarchy_builds()
    );
    assert!(removed_fields
        .delta_from(&added_fields)
        .removed_fields()
        .iter()
        .any(|field| field.component_type_path() == DYNAMIC_TYPE_PATH
            && field.field_name() == "coverage"));
}

#[test]
fn removing_a_parent_invalidates_the_orphaned_child_field_artifact() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let initial_fields = world
        .inspection_fields_artifact(child)
        .expect("child should expose reflected fields");

    assert!(world.remove_entity(parent));

    let current_fields = world
        .inspection_fields_artifact(child)
        .expect("orphaned child should remain inspectable");
    let parent_field = current_fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path.ends_with("::Hierarchy") && field.field_name == "parent"
        })
        .expect("child should expose its hierarchy parent field");
    let field_delta = current_fields.delta_from(&initial_fields);

    assert_eq!(current_fields.generation(), world.world_generation());
    assert_eq!(
        parent_field.value,
        zircon_runtime_interface::reflect::ReflectedValue::Entity(None)
    );
    assert!(field_delta.changed_fields().iter().any(|field| {
        field.component_type_path.ends_with("::Hierarchy")
            && field.field_name == "parent"
            && field.value == zircon_runtime_interface::reflect::ReflectedValue::Entity(None)
    }));
}

#[test]
fn active_parent_mutation_invalidates_descendant_field_artifacts() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let initial_fields = world
        .inspection_fields_artifact(child)
        .expect("child should expose reflected fields");

    assert!(world.set_active_self(parent, false).unwrap());

    let current_fields = world
        .inspection_fields_artifact(child)
        .expect("inactive descendant should remain inspectable");
    let active_field = current_fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path.ends_with("::ActiveInHierarchy")
                && field.field_name == "value"
        })
        .expect("child should expose its derived active state");
    let field_delta = current_fields.delta_from(&initial_fields);

    assert_eq!(current_fields.generation(), world.world_generation());
    assert_eq!(
        active_field.value,
        zircon_runtime_interface::reflect::ReflectedValue::Bool(false)
    );
    assert!(field_delta.changed_fields().iter().any(|field| {
        field.component_type_path.ends_with("::ActiveInHierarchy")
            && field.field_name == "value"
            && field.value == zircon_runtime_interface::reflect::ReflectedValue::Bool(false)
    }));
}

#[test]
fn unrelated_transform_change_reuses_the_focused_field_payload() {
    let mut world = World::empty();
    let focused = world.spawn_node(NodeKind::Empty);
    let changed = world.spawn_node(NodeKind::Empty);
    let initial_fields = world
        .inspection_fields_artifact(focused)
        .expect("focused entity should expose reflected fields");
    let initial_diagnostics = world.inspection_artifact_diagnostics();

    assert!(world
        .update_transform(
            changed,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .expect("unrelated transform update should succeed"));
    let current_fields = world
        .inspection_fields_artifact(focused)
        .expect("focused entity should remain inspectable");

    assert_eq!(current_fields.generation(), world.world_generation());
    assert_eq!(
        world
            .inspection_artifact_diagnostics()
            .focused_field_builds(),
        initial_diagnostics.focused_field_builds()
    );
    let field_delta = current_fields.delta_from(&initial_fields);
    assert!(field_delta.changed_fields().is_empty());
    assert!(field_delta.removed_fields().is_empty());
}

#[test]
fn focused_field_artifact_reuses_its_arc_and_reports_property_changes() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    let other_entity = world.spawn_node(NodeKind::Cube);

    let first = world.inspection_fields_artifact(entity).unwrap();
    let stable = world.inspection_fields_artifact(entity).unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &stable));
    let initial_diagnostics = world.inspection_artifact_diagnostics();
    world.inspection_fields_artifact(other_entity).unwrap();
    let changed_selection_diagnostics = world.inspection_artifact_diagnostics();
    assert_eq!(
        changed_selection_diagnostics.hierarchy_builds(),
        initial_diagnostics.hierarchy_builds()
    );
    assert_eq!(
        changed_selection_diagnostics.focused_field_builds(),
        initial_diagnostics.focused_field_builds() + 1
    );
    let name_field = first
        .fields()
        .iter()
        .find(|field| field.component_type_path.ends_with("::Name"))
        .expect("empty nodes expose a reflected Name field");

    world.rename_node(entity, "Renamed").unwrap();
    let changed = world.inspection_fields_artifact(entity).unwrap();
    let delta = changed.delta_from(&first);

    assert!(!std::sync::Arc::ptr_eq(&first, &changed));
    assert_eq!(delta.entity(), entity);
    assert_eq!(delta.previous_generation(), first.generation());
    assert_eq!(delta.generation(), changed.generation());
    assert!(delta.changed_fields().iter().any(|field| {
        field.component_type_path == name_field.component_type_path
            && field.field_name == name_field.field_name
    }));
    assert!(delta.removed_fields().is_empty());
}

#[test]
fn stable_thousand_node_inspection_reuses_the_initial_hierarchy_build() {
    const NODE_COUNT: usize = 1_000;

    let mut world = World::empty();
    for _ in 0..NODE_COUNT {
        world.spawn_node(NodeKind::Empty);
    }

    let initial = world.inspection_artifact();
    let initial_diagnostics = world.inspection_artifact_diagnostics();
    assert_eq!(initial.hierarchy_rows().len(), NODE_COUNT);
    assert_eq!(initial_diagnostics.hierarchy_builds(), 1);
    assert_eq!(
        initial_diagnostics.hierarchy_rows_built(),
        NODE_COUNT as u64
    );

    for _ in 0..8 {
        let stable = world.inspection_artifact();
        assert!(std::sync::Arc::ptr_eq(&initial, &stable));
    }
    assert_eq!(world.inspection_artifact_diagnostics(), initial_diagnostics);
}

#[test]
fn inspection_artifact_delta_is_entity_addressable() {
    let mut world = World::empty();
    let renamed = world.spawn_node(NodeKind::Empty);
    let removed = world.spawn_node(NodeKind::Empty);
    let initial = world.inspection_artifact();

    world.rename_node(renamed, "Renamed").unwrap();
    assert!(world.remove_entity(removed));
    let added = world.spawn_node(NodeKind::Cube);
    let current = world.inspection_artifact();
    let delta = current.delta_from(&initial);

    assert_eq!(delta.previous_generation(), initial.generation());
    assert_eq!(delta.generation(), current.generation());
    assert_eq!(
        delta
            .added_rows()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![added]
    );
    assert_eq!(
        delta
            .changed_rows()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![renamed]
    );
    assert_eq!(delta.removed_entities(), &[removed]);
}

#[test]
fn published_inspection_delta_reuses_the_immediately_preceding_generation() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    let initial = world.inspection_artifact();

    world.rename_node(entity, "Renamed").unwrap();
    let current = world.inspection_artifact();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("the adjacent artifact generation should carry its published delta");

    assert_eq!(delta.previous_generation(), initial.generation());
    assert_eq!(delta.generation(), current.generation());
    assert!(delta.added_rows().is_empty());
    assert_eq!(
        delta
            .changed_rows()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
    assert!(delta.removed_entities().is_empty());
}

#[test]
fn hundred_thousand_node_transform_change_publishes_only_the_target_field_delta() {
    const NODE_COUNT: usize = 100_000;

    let mut world = World::empty();
    let mut target = 0;
    for index in 0..NODE_COUNT {
        let entity = world.spawn_node(NodeKind::Empty);
        if index == NODE_COUNT / 2 {
            target = entity;
        }
    }
    let initial = world.inspection_artifact();
    let initial_fields = world
        .inspection_fields_artifact(target)
        .expect("target should expose reflected fields");
    assert_eq!(initial.hierarchy_rows().len(), NODE_COUNT);

    world
        .update_transform(
            target,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    let current = world.inspection_artifact();
    let delta = current
        .published_delta_from(initial.generation())
        .expect("the adjacent generation should retain the bounded notification delta");

    assert!(delta.added_rows().is_empty());
    assert!(delta.changed_rows().is_empty());
    assert!(delta.removed_entities().is_empty());
    let current_fields = world
        .inspection_fields_artifact(target)
        .expect("target should retain reflected fields");
    let fields_delta = current_fields.delta_from(&initial_fields);
    assert_eq!(fields_delta.entity(), target);
    assert!(!fields_delta.changed_fields().is_empty());
    assert!(fields_delta.removed_fields().is_empty());
    let diagnostics = world.inspection_artifact_diagnostics();
    assert_eq!(diagnostics.hierarchy_builds(), 1);
    assert_eq!(diagnostics.hierarchy_rows_built(), NODE_COUNT as u64);
    assert_eq!(diagnostics.focused_field_builds(), 2);
}

#[test]
fn inspection_projection_avoids_redundant_focus_and_field_key_work() {
    let source = include_str!("snapshot.rs");

    assert!(source.contains("build_hierarchy_rows(world, focused_entity)"));
    assert!(!source.contains("for row in &mut hierarchy_rows"));
    assert!(source.contains("HashMap<Option<EntityId>, Vec<EntityId>>"));
    assert!(source.contains("field.field_name.as_str()"));
    assert!(!source.contains("field.field_name.clone(), field"));
}

#[test]
fn subtree_hash_propagates_descendant_name_changes_without_touching_unrelated_roots() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    let unrelated = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(root)).unwrap();

    let before = hashes_by_entity(&world);
    world
        .insert(child, Name("Renamed Child".to_string()))
        .unwrap();
    let after = hashes_by_entity(&world);

    assert_ne!(after[&child], before[&child]);
    assert_ne!(after[&root], before[&root]);
    assert_eq!(after[&unrelated], before[&unrelated]);
    assert_eq!(hashes_by_entity(&world), after);
}

#[test]
fn subtree_hash_tracks_parent_child_identity_without_changing_the_moved_subtree() {
    let mut world = World::empty();
    let first_parent = world.spawn_node(NodeKind::Empty);
    let second_parent = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    let before = hashes_by_entity(&world);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    let after = hashes_by_entity(&world);

    assert_ne!(after[&first_parent], before[&first_parent]);
    assert_ne!(after[&second_parent], before[&second_parent]);
    assert_eq!(after[&child], before[&child]);
}

#[test]
fn subtree_hash_encodes_cycle_edges_even_when_the_child_was_already_visited() {
    let mut source = World::empty();
    let first = source.spawn_node(NodeKind::Empty);
    let second = source.spawn_node(NodeKind::Empty);
    let missing_parent = second + 100;

    let cycle =
        world_with_serialized_parents(&source, &[(first, Some(second)), (second, Some(first))]);
    let broken_cycle = world_with_serialized_parents(
        &source,
        &[(first, Some(missing_parent)), (second, Some(first))],
    );

    assert_eq!(
        cycle
            .inspect_hierarchy()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_eq!(
        broken_cycle
            .inspect_hierarchy()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_ne!(hashes_by_entity(&cycle), hashes_by_entity(&broken_cycle));
}

#[test]
fn deep_hierarchy_inspection_is_iterative_and_deterministic() {
    const DEPTH: usize = 5_000;

    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let mut parent = root;
    for _ in 1..DEPTH {
        let child = world.spawn_node(NodeKind::Empty);
        world.set_parent_checked(child, Some(parent)).unwrap();
        parent = child;
    }

    let first = world.inspect_hierarchy();
    let second = world.inspect_hierarchy();

    assert_eq!(first.len(), DEPTH);
    assert_eq!(first, second);
    assert_eq!(first.first().map(|row| row.depth), Some(0));
    assert_eq!(first.last().map(|row| row.depth), Some((DEPTH - 1) as u32));
}
