#[test]
fn archetype_index_hot_paths_use_direct_branches_without_adapter_chains() {
    let source = include_str!("../ecs/archetype/index.rs");
    let record_source = include_str!("../ecs/archetype/record.rs");
    let signature_lookup = source
        .split("pub fn signature(&self, id: ArchetypeId) -> Option<&ArchetypeSignature>")
        .nth(1)
        .and_then(|text| text.split("pub fn entities(&self, id: ArchetypeId)").next())
        .expect("read ArchetypeIndex::signature body");
    let entities_lookup = source
        .split("pub fn entities(&self, id: ArchetypeId) -> Option<&[EntityId]>")
        .nth(1)
        .and_then(|text| text.split("pub fn id_or_insert").next())
        .expect("read ArchetypeIndex::entities body");
    let move_entity = source
        .split("pub fn move_entity(")
        .nth(1)
        .and_then(|text| text.split("pub fn matching_archetypes").next())
        .expect("read ArchetypeIndex::move_entity body");
    let matching_archetypes = source
        .split("pub fn matching_archetypes(")
        .nth(1)
        .and_then(|text| text.split("fn shortest_required_archetype_ids").next())
        .expect("read ArchetypeIndex::matching_archetypes body");
    let shortest_required = source
        .split("fn shortest_required_archetype_ids(&self, required: &[ComponentId])")
        .nth(1)
        .and_then(|text| text.split("fn archetype_matches_required_without").next())
        .expect("read ArchetypeIndex::shortest_required_archetype_ids body");
    let match_helper = source
        .split("fn archetype_matches_required_without(")
        .nth(1)
        .and_then(|text| text.split("fn add_entity_to").next())
        .expect("read ArchetypeIndex::archetype_matches_required_without body");
    let add_entity = source
        .split("fn add_entity_to(&mut self, id: ArchetypeId, entity: EntityId) -> usize")
        .nth(1)
        .and_then(|text| text.split("fn remove_entity_from").next())
        .expect("read ArchetypeIndex::add_entity_to body");
    let remove_entity = source
        .split("fn remove_entity_from(")
        .nth(1)
        .and_then(|text| text.split("fn index_signature_components").next())
        .expect("read ArchetypeIndex::remove_entity_from body");
    let swap_remove_entity = record_source
        .split("pub(super) fn swap_remove_entity(")
        .nth(1)
        .and_then(|text| text.split("\n    }\n}").next())
        .expect("read ArchetypeRecord::swap_remove_entity body");
    let entity_row = source
        .split("fn entity_row(entities: &[EntityId], entity: EntityId) -> Option<usize>")
        .nth(1)
        .and_then(|text| text.split("impl Default for ArchetypeIndex").next())
        .expect("read entity_row helper");

    assert!(signature_lookup.contains("let record = self.records.get(id.index())?;"));
    assert!(signature_lookup.contains("Some(record.signature())"));
    assert!(!signature_lookup.contains(".map(ArchetypeRecord::signature)"));
    assert!(entities_lookup.contains("let record = self.records.get(id.index())?;"));
    assert!(entities_lookup.contains("Some(record.entities())"));
    assert!(!entities_lookup.contains(".map(ArchetypeRecord::entities)"));

    assert!(move_entity.contains("let swapped_entity = if let Some(id) = previous"));
    assert!(move_entity.contains("self.remove_entity_from(id, entity)"));
    assert!(!move_entity.contains("previous.and_then"));

    assert!(matching_archetypes.contains("self.shortest_required_archetype_ids(required)"));
    assert!(matching_archetypes.contains("if ids.is_empty()"));
    assert!(matching_archetypes.contains("return Vec::new();"));
    assert!(matching_archetypes.contains("let mut matches = Vec::with_capacity(ids.len());"));
    assert!(matching_archetypes.contains("for id in ids"));
    assert!(matching_archetypes
        .contains("if self.archetype_matches_required_without(*id, required, without)"));
    assert!(matching_archetypes.contains("matches.push(*id);"));
    assert!(matching_archetypes.contains("return matches;"));
    assert!(
        matching_archetypes.contains("let mut matches = Vec::with_capacity(self.records.len());")
    );
    assert!(matching_archetypes.contains("for record in &self.records"));
    assert!(matching_archetypes.contains("let id = record.id();"));
    assert!(matching_archetypes.contains("matches.push(id);"));
    assert!(matching_archetypes.contains("matches"));
    assert!(!matching_archetypes.contains("self.shortest_required_component(required)"));
    assert!(!matching_archetypes.contains("let Some(ids) = self.by_component.get(&component_id)"));
    assert!(!matching_archetypes.contains("ids.clone()"));
    assert!(!matching_archetypes.contains("candidates.retain("));
    assert!(!source.contains("fn all_archetype_ids"));
    assert!(!matching_archetypes.contains(".min_by_key("));
    assert!(!matching_archetypes.contains(".map_or(0, Vec::len)"));
    assert!(!matching_archetypes.contains(".cloned()"));
    assert!(!matching_archetypes.contains(".unwrap_or_default()"));
    assert!(!matching_archetypes.contains(".is_some_and("));

    assert!(shortest_required.contains("for component_id in required"));
    assert!(shortest_required.contains("match self.by_component.get(component_id)"));
    assert!(shortest_required.contains("Some(ids) => ids.as_slice()"));
    assert!(shortest_required.contains("None => return Some(&[])"));
    assert!(shortest_required.contains("let candidate_len = ids.len();"));
    assert!(shortest_required.contains("selected = Some(ids);"));
    assert!(shortest_required.contains("if candidate_len == 0"));
    assert!(!shortest_required.contains("selected = Some(*component_id);"));
    assert!(match_helper.contains("let Some(record) = self.records.get(id.index()) else"));
    assert!(match_helper.contains("for component_id in required"));
    assert!(match_helper.contains("for component_id in without"));
    assert!(!match_helper.contains(".all(|component_id|"));

    assert!(add_entity.contains("entity_row(record.entities(), entity)"));
    assert!(remove_entity.contains("entity_row(record.entities(), entity)?"));
    assert!(remove_entity.contains("record.swap_remove_entity(row, entity)"));
    assert!(!remove_entity.contains(".position("));
    assert!(!remove_entity.contains(".then(||"));
    assert!(swap_remove_entity.contains("let last_row = self.entities.len() - 1;"));
    assert!(swap_remove_entity.contains("debug_assert_eq!(removed, entity);"));
    assert!(swap_remove_entity.contains("if row != last_row"));
    assert!(swap_remove_entity.contains("Some((self.entities[row], row))"));
    assert!(entity_row.contains("while row < entities.len()"));
    assert!(entity_row.contains("if entities[row] == entity"));
    assert!(!entity_row.contains(".iter().position("));
}
