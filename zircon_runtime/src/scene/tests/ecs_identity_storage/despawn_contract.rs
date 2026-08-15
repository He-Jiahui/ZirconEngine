#[test]
fn world_despawn_uses_known_archetype_location_without_full_rebuild() {
    let hierarchy = include_str!("../../world/hierarchy.rs");
    let remove_entity = hierarchy
        .split("pub fn remove_entity(&mut self, entity: EntityId) -> SceneResult<()>")
        .nth(1)
        .and_then(|text| text.split("pub fn subtree_records").next())
        .expect("read World::remove_entity body");

    assert!(
        remove_entity.contains("self.remove_entity_from_archetype(entity);")
            && !remove_entity.contains("self.refresh_stable_entity_locations();"),
        "despawn must remove from the known archetype row and repair only the swapped entity instead of rebuilding every archetype"
    );
}

#[test]
fn world_despawn_removes_only_components_in_the_current_archetype_signature() {
    let hierarchy = include_str!("../../world/hierarchy.rs");
    let remove_entity = hierarchy
        .split("pub fn remove_entity(&mut self, entity: EntityId) -> SceneResult<()>")
        .nth(1)
        .and_then(|text| text.split("pub fn subtree_records").next())
        .expect("read World::remove_entity body");

    assert!(
        remove_entity.contains("let component_ids = self.entity_archetype_component_ids(entity);")
            && remove_entity.contains(".remove_entity_components(internal, &component_ids);")
            && !remove_entity.contains("component_ids_for_entity(internal)")
            && !remove_entity.contains("component_storage.remove_entity(internal)"),
        "despawn must use the current archetype signature instead of scanning every registered component storage"
    );
}
