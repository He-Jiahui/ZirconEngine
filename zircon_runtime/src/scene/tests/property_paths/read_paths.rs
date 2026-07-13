use super::*;

#[test]
fn world_property_reads_compare_normalized_segments_without_entry_vector_allocation() {
    let read_source = include_str!("../../world/property_access/read.rs");
    let entries_source = include_str!("../../world/property_access/entries.rs");
    let value_conversion_source = include_str!("../../world/property_access/value_conversion.rs");

    assert!(read_source.contains("let target_component = property_path.component();"));
    assert!(read_source.contains("let target_segments = property_path.property_segments();"));
    assert!(read_source
        .contains("self.property_entry_value(entity, target_component, target_segments)"));
    assert!(read_source.contains("return Ok(value);"));
    assert!(read_source
        .contains("if let Some(value) = self.dynamic_component_property(entity, property_path)"));
    assert!(read_source.contains(") -> SceneResult<ScenePropertyValue>"));
    assert!(read_source.contains("SceneError::PropertyUnavailable"));
    assert!(read_source.contains("property_path: property_path.to_string()"));
    assert!(entries_source.contains("pub(super) fn property_entry_value("));
    assert!(entries_source.contains("self.visit_property_entries(entity, false"));
    assert!(entries_source.contains("fn visit_property_entries<"));
    assert!(entries_source.contains("macro_rules! push_entry"));
    assert!(entries_source.contains("if !visitor($path, $value, $animatable)"));
    assert!(entries_source.contains("fn property_path_literal_matches_normalized("));
    assert!(entries_source.contains("path.split_once('.')"));
    assert!(entries_source.contains("fn property_literal_segments_match_normalized("));
    assert!(entries_source.contains("for segment in segments.split('.')"));
    assert!(entries_source.contains("normalized_identifier_matches(component, target_component)"));
    assert!(entries_source
        .contains("!normalized_identifier_matches(segment, &target_segments[target_index])"));
    assert!(entries_source.contains("if include_dynamic {"));
    assert!(value_conversion_source.contains("pub(super) fn normalized_identifier_matches("));
    assert!(value_conversion_source.contains("let mut value_chars = value.chars();"));
    assert!(value_conversion_source.contains("let mut target_chars = target.chars();"));
    assert!(value_conversion_source.contains("next_normalized_identifier_char"));
    assert!(!read_source.contains("let entries = self.property_entries(entity);"));
    assert!(!read_source.contains("entries\n            .into_iter()"));
    assert!(!read_source.contains("fn property_path_matches_normalized("));
    assert!(!read_source.contains("fn property_segments_match_normalized("));
    assert!(!entries_source
        .contains("let mut push = |path: &str, value: ScenePropertyValue, animatable: bool|"));
    assert!(!read_source.contains("use super::value_conversion::normalized_identifier;"));
    assert!(!read_source.contains("let target_component = normalized_identifier("));
    assert!(!read_source
        .contains(".or_else(|| self.dynamic_component_property(entity, property_path))"));
    assert!(!read_source.contains(".ok_or_else(||"));
    assert!(!read_source.contains(
        ".property_segments()\n                        .iter()\n                        .map(|segment| normalized_identifier(segment))\n                        .collect::<Vec<_>>()"
    ));
    assert!(!read_source.contains(".map(|segment| normalized_identifier(segment))"));
    assert!(!read_source.contains(".collect::<Vec<_>>()"));
    assert!(!read_source
        .contains("normalized_identifier(property_path.component()) == target_component"));
    assert!(
        !read_source.contains("normalized_identifier(&segments[index]) != target_segments[index]")
    );
    assert!(!read_source.contains(".zip(target_segments)"));
    assert!(!read_source
        .contains(".all(|(segment, target)| normalized_identifier(segment) == *target)"));
}

#[test]
fn component_property_path_constructor_pre_sizes_raw_path_buffer() {
    let nested_path = ComponentPropertyPath::new(
        "MeshRenderer",
        vec!["morph_weights".to_string(), "1".to_string()],
    )
    .expect("component property path should accept nested property segments");
    assert_eq!(nested_path.as_str(), "MeshRenderer.morph_weights.1");

    let source = include_str!("../../../core/framework/scene/entity_path.rs");
    let constructor = source
        .split("impl ComponentPropertyPath")
        .nth(1)
        .and_then(|text| text.split("pub fn parse").next())
        .expect("read ComponentPropertyPath constructor body");
    let raw_helper = source
        .split("fn component_property_path_raw")
        .nth(1)
        .and_then(|text| {
            text.split("impl fmt::Display for ComponentPropertyPath")
                .next()
        })
        .expect("read ComponentPropertyPath raw helper body");
    let validation_helper = source
        .split("fn validated_property_segments_len")
        .nth(1)
        .and_then(|text| text.split("fn component_property_path_raw").next())
        .expect("read ComponentPropertyPath validation helper body");

    assert!(
        constructor.contains("let property_len = validated_property_segments_len(&property_segments)?;")
            && constructor.contains(
                "raw: component_property_path_raw(&component, &property_segments, property_len),"
            )
            && validation_helper.contains("let mut property_len = 0;")
            && validation_helper.contains("for segment in property_segments")
            && validation_helper.contains("if segment.trim().is_empty()")
            && validation_helper.contains("property_len += segment.len();")
            && raw_helper.contains("property_len: usize")
            && raw_helper.contains("let mut raw = String::with_capacity(component.len() + property_len + property_segments.len());")
            && raw_helper.contains("raw.push_str(component);")
            && raw_helper.contains("for segment in property_segments")
            && raw_helper.contains("raw.push('.');")
            && raw_helper.contains("raw.push_str(segment);")
            && !constructor.contains(".iter()\n            .any(|segment| segment.trim().is_empty())")
            && !raw_helper.contains("property_segments.iter().map(String::len).sum::<usize>()")
            && !constructor.contains("property_segments.join(\".\")")
            && !constructor.contains("format!(\"{}.{}\", component, property_segments.join(\".\"))"),
        "ComponentPropertyPath::new must validate property segments and compute the raw-path capacity in one pass before pushing the path"
    );
}

#[test]
fn world_entity_path_resolution_compares_target_segments_directly() {
    let path_resolution_source = include_str!("../../world/property_access/path_resolution.rs");
    let old_entity_path_lookup = ["resolve", "entity", "path"].join("_");

    assert!(path_resolution_source
        .contains("pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId>"));
    assert!(!path_resolution_source.contains(&format!("pub fn {old_entity_path_lookup}")));
    assert!(path_resolution_source.contains("let target_segments = path.segments();"));
    assert!(path_resolution_source.contains("let mut entity_index = 0;"));
    assert!(path_resolution_source.contains("while entity_index < self.entities.len()"));
    assert!(path_resolution_source.contains("let entity = self.entities[entity_index];"));
    assert!(path_resolution_source
        .contains("if self.entity_matches_path_segments(entity, target_segments)"));
    assert!(path_resolution_source.contains("return Some(entity);"));
    assert!(path_resolution_source.contains("entity_index += 1;"));
    assert!(path_resolution_source.contains("\n        None\n"));
    assert!(path_resolution_source
        .contains("Vec::with_capacity(self.entity_path_segment_capacity(entity))"));
    assert!(path_resolution_source
        .contains("fn entity_path_segment_capacity(&self, entity: EntityId) -> usize"));
    assert!(path_resolution_source.contains("capacity += 1;"));
    assert!(path_resolution_source.contains(
        "fn entity_matches_path_segments(&self, entity: EntityId, target_segments: &[String])"
    ));
    assert!(path_resolution_source.contains("let mut segment_index = target_segments.len();"));
    assert!(
        path_resolution_source.contains("if segment_index == 0 {\n                return false;")
    );
    assert!(path_resolution_source.contains("segment_index -= 1;"));
    assert!(path_resolution_source
        .contains("let Some(segment) = self.path_segment_for_entity(current) else"));
    assert!(path_resolution_source.contains("if segment != target_segments[segment_index]"));
    assert!(path_resolution_source.contains("segment_index == 0"));
    assert!(path_resolution_source.contains("fn entity_has_duplicate_path_name("));
    assert!(path_resolution_source.contains("let mut candidate_index = 0;"));
    assert!(path_resolution_source.contains("while candidate_index < self.entities.len()"));
    assert!(path_resolution_source.contains("let candidate = self.entities[candidate_index];"));
    assert!(path_resolution_source.contains("candidate_index += 1;"));
    assert!(path_resolution_source
        .contains("if candidate == entity || self.parent_of(candidate) != parent"));
    assert!(path_resolution_source
        .contains("let Some(candidate_name) = self.names.get(&candidate) else"));
    assert!(path_resolution_source.contains("if candidate_name.0.trim() == name"));
    assert!(path_resolution_source.contains("return true;"));
    assert!(path_resolution_source.contains("\n        false\n"));
    assert!(!path_resolution_source.contains(
        "self.entity_path(*entity)\n                .as_ref()\n                .is_some_and(|candidate| candidate == path)"
    ));
    assert!(!path_resolution_source.contains("let mut segments = Vec::new();"));
    assert!(!path_resolution_source.contains("let duplicate_count = self"));
    assert!(!path_resolution_source.contains(".count();\n        Some(if duplicate_count > 1"));
    assert!(!path_resolution_source.contains(".filter(|candidate| *candidate != entity)"));
    assert!(!path_resolution_source.contains(".any(|candidate| {"));
    assert!(!path_resolution_source.contains(".find(|entity| self.entity_matches_path_segments"));
    assert!(!path_resolution_source.contains("for candidate in self.entities.iter().copied()"));
    assert!(!path_resolution_source
        .contains("self.entities\n            .iter()\n            .copied()"));
}

#[test]
fn world_property_entries_pre_size_projection_vector() {
    let entries_source = include_str!("../../world/property_access/entries.rs");
    let physics_entries_source = include_str!("../../world/property_access/entries/physics.rs");
    let collider_shape_entries_source =
        include_str!("../../world/property_access/entries/collider_shape.rs");

    assert!(
        entries_source.contains("Vec::with_capacity(self.property_entry_capacity_hint(entity))")
    );
    assert!(entries_source.contains("push_entry!("));
    assert!(entries_source.contains("\"Hierarchy.parent\""));
    assert!(entries_source.contains("ScenePropertyValue::Entity(self.parent_of(entity))"));
    assert!(entries_source.contains("fn property_entry_capacity_hint(&self, entity: EntityId)"));
    assert!(entries_source.contains("capacity += 10 + mesh.morph_weights.len();"));
    assert!(entries_source.contains("self.visit_physics_property_entries(entity, &mut visitor)"));
    assert!(
        entries_source.contains("capacity += self.physics_property_entry_capacity_hint(entity);")
    );
    assert!(physics_entries_source.contains("pub(super) fn visit_physics_property_entries"));
    assert!(physics_entries_source.contains("pub(super) fn physics_property_entry_capacity_hint"));
    assert!(physics_entries_source.contains("capacity += 17;"));
    assert!(physics_entries_source.contains("if let Some(collider) = self.colliders.get(&entity)"));
    assert!(physics_entries_source
        .contains("capacity += collider_shape_property_entry_capacity(&collider.shape);"));
    assert!(collider_shape_entries_source.contains(
        "pub(super) fn collider_shape_property_entry_capacity(shape: &ColliderShape) -> usize"
    ));
    assert!(collider_shape_entries_source
        .contains("3 + collider_shape_property_entry_capacity(child_shape.as_ref())"));
    assert!(entries_source.contains("capacity += 2 + player.parameters.len();"));
    assert!(entries_source.contains("capacity += 3 + player.parameters.len();"));
    assert!(entries_source.contains("match &player.active_state"));
    assert!(entries_source.contains("Some(active_state) => active_state.clone()"));
    assert!(entries_source.contains("None => String::new()"));
    assert!(entries_source.contains("for value in properties.values() {"));
    assert!(entries_source.contains("if dynamic_scene_value_is_projectable(value) {"));
    assert!(entries_source.contains("capacity += 1;\n                    }"));
    assert!(entries_source.contains("fn dynamic_scene_value_is_projectable(value: &Value)"));
    assert!(entries_source.contains("use std::fmt::Write as _;"));
    assert!(entries_source.contains(
        "const MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX: &str = \"MeshRenderer.morph_weights.\";"
    ));
    assert!(entries_source.contains("fn mesh_renderer_morph_weight_path(index: usize) -> String"));
    assert!(entries_source.contains("fn decimal_digit_count(mut value: usize) -> usize"));
    assert!(entries_source.contains("let mut morph_weight_index = 0;"));
    assert!(entries_source.contains("while morph_weight_index < mesh.morph_weights.len()"));
    assert!(
        entries_source.contains("let path = mesh_renderer_morph_weight_path(morph_weight_index);")
    );
    assert!(entries_source.contains("let weight = mesh.morph_weights[morph_weight_index];"));
    assert!(entries_source.contains("ScenePropertyValue::Scalar(weight)"));
    assert!(entries_source.contains("morph_weight_index += 1;"));
    assert!(
        entries_source.contains("let prefix_len = MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX.len();")
    );
    assert!(
        entries_source.contains("String::with_capacity(prefix_len + decimal_digit_count(index))")
    );
    assert!(entries_source.contains("path.push_str(MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX);"));
    assert!(entries_source
        .contains("write!(&mut path, \"{index}\").expect(\"writing to a String cannot fail\");"));
    assert!(!entries_source.contains("let mut entries = Vec::new();"));
    assert!(
        !entries_source.contains("for (index, weight) in mesh.morph_weights.iter().enumerate()")
    );
    assert!(!entries_source.contains("&format!(\"MeshRenderer.morph_weights.{index}\")"));
    assert!(!entries_source.contains("player.active_state.clone().unwrap_or_default()"));
    assert!(!entries_source.contains(
        ".values()\n                    .filter(|value| dynamic_scene_value_is_projectable(value))\n                    .count()"
    ));
    assert!(!entries_source.contains(
        "if self.contains_entity(entity) {\n            push(\n                \"Hierarchy.parent\""
    ));
}

#[test]
fn world_property_dynamic_json_number_projection_uses_direct_branches() {
    let entries_source = include_str!("../../world/property_access/entries.rs");
    let json_projection_source = entries_source
        .split("fn dynamic_scene_value_from_json(value: &Value)")
        .nth(1)
        .and_then(|text| text.split("fn dynamic_scene_value_is_projectable").next())
        .expect("read dynamic_scene_value_from_json helper body");

    assert!(json_projection_source.contains("if let Some(value) = value.as_i64()"));
    assert!(json_projection_source.contains("return Some(ScenePropertyValue::Integer(value));"));
    assert!(json_projection_source.contains("if let Some(value) = value.as_u64()"));
    assert!(json_projection_source.contains("return Some(ScenePropertyValue::Unsigned(value));"));
    assert!(json_projection_source.contains("if let Some(value) = value.as_f64()"));
    assert!(json_projection_source.contains("return Some(ScenePropertyValue::Scalar(value as _));"));
    assert!(!json_projection_source.contains(".map(ScenePropertyValue::Integer)"));
    assert!(!json_projection_source.contains(".or_else(||"));
}
