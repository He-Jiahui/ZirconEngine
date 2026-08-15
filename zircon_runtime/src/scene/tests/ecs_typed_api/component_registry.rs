use super::*;

#[test]
fn fixed_component_setters_and_dynamic_components_share_component_id_presence() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Runtime Body".to_string()),)).unwrap();
    let rigid_body_id = world.component_id::<RigidBodyComponent>();

    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();

    assert!(world.contains_component_id(entity, rigid_body_id));
    assert_eq!(
        world.get::<RigidBodyComponent>(entity),
        Some(&RigidBodyComponent::default())
    );

    world.set_rigid_body(entity, None).unwrap();

    assert!(!world.contains_component_id(entity, rigid_body_id));
    assert_eq!(world.get::<RigidBodyComponent>(entity), None);

    world
        .register_component_type(ComponentTypeDescriptor {
            type_id: "weather.cloud".to_string(),
            plugin_id: "weather".to_string(),
            display_name: "Cloud".to_string(),
            properties: vec![ComponentPropertyDescriptor {
                name: "density".to_string(),
                value_type: "number".to_string(),
                editable: true,
            }],
        })
        .unwrap();

    world
        .set_dynamic_component(entity, "weather.cloud", json!({ "density": 0.75 }))
        .unwrap();
    let dynamic_component_id = world
        .registered_dynamic_component_id("weather.cloud")
        .unwrap();

    assert!(world.contains_component_id(entity, dynamic_component_id));
    assert_eq!(
        world.dynamic_component(entity, "weather.cloud"),
        Some(&json!({ "density": 0.75 }))
    );

    world
        .remove_dynamic_component(entity, "weather.cloud")
        .unwrap();

    assert!(!world.contains_component_id(entity, dynamic_component_id));
}

#[test]
fn world_registry_rebuild_defers_rows_to_the_complete_projection_owner() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    );
    let rebuild = source
        .split("pub(super) fn reset_archetype_index_for_projection")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn ensure_archetype").next())
        .expect("read archetype index projection reset body");

    assert!(rebuild.contains("self.archetype_index = Default::default();"));
    assert!(rebuild.contains("self.stable_query_order.clear_archetypes();"));
    assert!(!rebuild.contains("append_empty_archetype_row"));
    assert!(!rebuild.contains("set_location"));
    assert!(!rebuild.contains("component_storage"));
}

#[test]
fn archetype_signatures_come_from_the_entity_archetype_record_without_storage_scans() {
    let identity_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    );
    let signature = identity_source
        .split("pub(super) fn entity_archetype_signature")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn archetype_assignment_count")
                .next()
        })
        .expect("read archetype signature body");

    assert!(
        signature.contains("self.archetype_location_for_entity(entity)?")
            && signature.contains("self.archetype_index.signature(archetype_id).cloned()")
            && !signature.contains("component_storage"),
        "entity archetype signatures must be read directly from the authoritative archetype record"
    );
}

#[test]
fn typed_component_presence_rebuild_aggregates_dynamic_ids_into_final_rows() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("projection_rebuild.rs"),
    );
    let rebuild = source
        .split("pub(super) fn rebuild_component_storage_projection_with_owned_components")
        .nth(1)
        .expect("read component projection rebuild body");

    assert!(
        rebuild.contains("let dynamic_component_type_ids = entities")
            && rebuild.contains("let mut rows = BTreeMap::new();")
            && rebuild.contains("self.stage_component_row_value_with_id(")
            && rebuild.contains("self.reset_archetype_index_for_projection();")
            && rebuild.contains("self.commit_rebuilt_component_row(entity, row);")
            && !rebuild.contains("self.rebuild_archetype_index();")
            && !rebuild.contains("self.commit_component_row(entity, row, false);")
            && !rebuild.contains("insert_dynamic_component_presence_without_archetype"),
        "projection rebuild must publish each aggregated final row directly without an intermediate empty-archetype row"
    );
}

#[test]
fn typed_world_presence_and_tracker_helpers_use_direct_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let contains_component_id = source
        .split("pub fn contains_component_id")
        .nth(1)
        .and_then(|text| text.split("pub fn contains_component<").next())
        .expect("read contains_component_id body");
    let contains_component = source
        .split("pub fn contains_component<")
        .nth(1)
        .and_then(|text| text.split("pub fn is_component_added").next())
        .expect("read contains_component body");
    let is_component_added = source
        .split("pub fn is_component_added")
        .nth(1)
        .and_then(|text| text.split("pub fn is_component_changed").next())
        .expect("read is_component_added body");
    let is_component_changed = source
        .split("pub fn is_component_changed")
        .nth(1)
        .and_then(|text| text.split("pub fn insert<T>").next())
        .expect("read is_component_changed body");
    let remove = source
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn resource_id").next())
        .expect("read remove body");
    let is_resource_added = source
        .split("pub fn is_resource_added")
        .nth(1)
        .and_then(|text| text.split("pub fn is_resource_changed").next())
        .expect("read is_resource_added body");
    let is_resource_changed = source
        .split("pub fn is_resource_changed")
        .nth(1)
        .and_then(|text| text.split("pub fn insert_resource").next())
        .expect("read is_resource_changed body");
    let get_resource_mut = source
        .split("pub fn get_resource_mut")
        .nth(1)
        .and_then(|text| text.split("pub fn remove_resource").next())
        .expect("read get_resource_mut body");

    assert!(
        contains_component_id.contains("let Some(internal) = self.internal_entity(entity) else")
            && contains_component_id.contains("return false;")
            && contains_component_id
                .contains("self.component_storage.contains(component_id, internal)")
            && !contains_component_id.contains(".is_some_and(")
            && contains_component
                .contains("let Some(component_id) = self.registered_component_id::<T>() else")
            && contains_component.contains("self.contains_component_id(entity, component_id)")
            && !contains_component.contains(".is_some_and(")
            && is_component_added
                .contains("let Some(ticks) = self.component_change_ticks::<T>(entity) else")
            && is_component_added
                .contains("ticks.is_added(crate::scene::ecs::ChangeTickWindow::new")
            && !is_component_added.contains(".is_some_and(")
            && is_component_changed
                .contains("let Some(ticks) = self.component_change_ticks::<T>(entity) else")
            && is_component_changed
                .contains("ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new")
            && !is_component_changed.contains(".is_some_and(")
            && remove.contains("if let Some(component_id) = component_id")
            && remove.contains("if self.contains_component_id(entity, component_id)")
            && !remove.contains("component_id\n                .is_some_and(")
            && !remove.contains("component_id.expect(\"checked component id must be present\")")
            && is_resource_added
                .contains("let Some(ticks) = self.resource_change_ticks::<T>() else")
            && is_resource_added
                .contains("ticks.is_added(crate::scene::ecs::ChangeTickWindow::new")
            && !is_resource_added.contains(".is_some_and(")
            && is_resource_changed
                .contains("let Some(ticks) = self.resource_change_ticks::<T>() else")
            && is_resource_changed
                .contains("ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new")
            && !is_resource_changed.contains(".is_some_and(")
            && get_resource_mut.contains(
                "let Some((resource, ticks, tick)) = self.resource_mut_with_ticks::<T>() else"
            )
            && get_resource_mut.contains("ticks.set_changed(tick);")
            && get_resource_mut.contains("Some(resource)")
            && !get_resource_mut.contains(".map(|(resource, ticks, tick)| resource)"),
        "typed world presence and tracker helpers must use direct branches instead of Option adapter predicates"
    );
}

#[test]
fn typed_world_component_insert_remove_route_dense_rows_and_sparse_values_explicitly() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let insert = source
        .split("pub fn insert<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get<T>").next())
        .expect("read typed component insert body");
    let remove = source
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn resource_id").next())
        .expect("read typed component remove body");

    assert!(
        insert.contains("let old = match T::STORAGE_TYPE")
            && insert.contains("StorageType::Table if was_present")
            && insert.contains("self.archetype_index")
            && insert.contains(".replace(")
            && insert.contains("StorageType::Table =>")
            && insert.contains("transition_entity_archetype_row(entity, signature, updates)")
            && insert.contains("StorageType::SparseSet =>")
            && insert.contains("self.component_storage.insert_at_tick(")
            && !insert.contains(".map_err(|error| error.to_string())")
            && remove.contains("let removed = match T::STORAGE_TYPE")
            && remove.contains("StorageType::Table =>")
            && remove.contains("transition_entity_archetype_row(entity, signature, updates)")
            && remove.contains("StorageType::SparseSet =>")
            && remove.contains("self.component_storage.remove::<T>(component_id, internal)?")
            && remove.contains("Some(ComponentRemoveResult { value }) => Some(value)")
            && remove.contains("None => None")
            && !remove.contains(".map_err(|error| error.to_string())")
            && !remove.contains(".map(|ComponentRemoveResult { value }| value)"),
        "typed component insert/remove must route dense rows through archetypes and sparse values through ComponentStorage"
    );
}

#[test]
fn dynamic_component_presence_updates_use_direct_result_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let insert_presence = source
        .split("pub(super) fn insert_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn remove_dynamic_component_presence")
                .next()
        })
        .expect("read dynamic component presence insert body");
    let remove_presence = source
        .split("pub(super) fn remove_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn rebuild_typed_component_presence")
                .next()
        })
        .expect("read dynamic component presence remove body");

    assert!(
        insert_presence.contains("let old = self.component_storage.insert_at_tick(")
            && insert_presence.contains("DynamicComponentPresence")
            && insert_presence.contains(")?;")
            && !insert_presence.contains(".map_err(|error| error.to_string())")
            && remove_presence.contains("let removed = self")
            && remove_presence
                .contains(".remove::<DynamicComponentPresence>(component_id, internal)")
            && remove_presence.contains(")?;")
            && !remove_presence.contains(".map_err(|error| error.to_string())"),
        "dynamic component presence updates must propagate typed SceneResult errors instead of map_err adapters"
    );
}
