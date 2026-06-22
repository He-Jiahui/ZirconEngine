use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::scene::components::{
    AnimationPlayerComponent, MeshRenderer, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

#[test]
fn world_resolves_entity_paths_and_mutates_component_properties() {
    let mut world = World::new();
    let root = world.spawn_node(NodeKind::Cube);
    world.rename_node(root, "Root").unwrap();

    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .update_transform(hero, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world
        .set_rigid_body(
            hero,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                mass: 2.5,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();

    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let mass_path = ComponentPropertyPath::parse("RigidBody.mass").unwrap();
    let weight_path = ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap();
    let render_queue_path = ComponentPropertyPath::parse("MeshRenderer.render_queue").unwrap();
    let material_queue_path = ComponentPropertyPath::parse("MeshRenderer.material_queue").unwrap();
    let order_path = ComponentPropertyPath::parse("MeshRenderer.order_in_layer").unwrap();
    let depth_bias_path = ComponentPropertyPath::parse("MeshRenderer.depth_bias").unwrap();
    let morph_weight_path = ComponentPropertyPath::parse("MeshRenderer.morph_weights.1").unwrap();

    assert_eq!(world.entity_path(hero), Some(entity_path.clone()));
    assert_eq!(world.get_entity_by_path(&entity_path), Some(hero));
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(2.5)
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.25)
    );
    assert_eq!(
        world.property(hero, &render_queue_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &material_queue_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &order_path).unwrap(),
        ScenePropertyValue::Integer(0)
    );
    assert_eq!(
        world.property(hero, &depth_bias_path).unwrap(),
        ScenePropertyValue::Scalar(0.0)
    );

    assert!(world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([4.0, 5.0, 6.0]),
        )
        .unwrap());
    assert!(world
        .set_property(hero, &mass_path, ScenePropertyValue::Scalar(5.5))
        .unwrap());
    assert!(world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert!(!world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert!(world
        .set_property(hero, &render_queue_path, ScenePropertyValue::Integer(2_450))
        .unwrap());
    assert!(world
        .set_property(hero, &material_queue_path, ScenePropertyValue::Integer(-12))
        .unwrap());
    assert!(world
        .set_property(hero, &order_path, ScenePropertyValue::Integer(14))
        .unwrap());
    assert!(!world
        .set_property(hero, &order_path, ScenePropertyValue::Integer(14))
        .unwrap());
    assert!(world
        .set_property(hero, &depth_bias_path, ScenePropertyValue::Scalar(-0.5))
        .unwrap());
    assert!(!world
        .set_property(hero, &depth_bias_path, ScenePropertyValue::Scalar(-0.5))
        .unwrap());
    assert!(world
        .set_property(hero, &morph_weight_path, ScenePropertyValue::Scalar(0.6))
        .unwrap());

    let node = world.find_node(hero).unwrap();
    assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(world.rigid_body(hero).unwrap().mass, 5.5);
    assert_eq!(world.animation_player(hero).unwrap().weight, 0.75);
    let mesh = world.get::<MeshRenderer>(hero).unwrap();
    assert_eq!(mesh.render_queue, 2_450);
    assert_eq!(mesh.material_queue, -12);
    assert_eq!(mesh.order_in_layer, 14);
    assert_eq!(mesh.depth_bias, -0.5);
    assert_eq!(mesh.morph_weights.as_slice(), &[0.0, 0.6]);
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([4.0, 5.0, 6.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(5.5)
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.75)
    );
    assert_eq!(
        world.property(hero, &render_queue_path).unwrap(),
        ScenePropertyValue::Integer(2_450)
    );
    assert_eq!(
        world.property(hero, &material_queue_path).unwrap(),
        ScenePropertyValue::Integer(-12)
    );
    assert_eq!(
        world.property(hero, &order_path).unwrap(),
        ScenePropertyValue::Integer(14)
    );
    assert_eq!(
        world.property(hero, &depth_bias_path).unwrap(),
        ScenePropertyValue::Scalar(-0.5)
    );
    assert_eq!(
        world.property(hero, &morph_weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.6)
    );
}

#[test]
fn world_entity_paths_suffix_duplicate_sibling_names() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Cube);
    world.rename_node(root, "Root").unwrap();
    let first = world.spawn_node(NodeKind::Mesh);
    world.rename_node(first, "Hero").unwrap();
    world.set_parent_checked(first, Some(root)).unwrap();
    let second = world.spawn_node(NodeKind::Mesh);
    world.rename_node(second, "Hero").unwrap();
    world.set_parent_checked(second, Some(root)).unwrap();

    let first_path = EntityPath::parse(&format!("Root/Hero#{first}")).unwrap();
    let second_path = EntityPath::parse(&format!("Root/Hero#{second}")).unwrap();

    assert_eq!(world.entity_path(first), Some(first_path.clone()));
    assert_eq!(world.entity_path(second), Some(second_path.clone()));
    assert_eq!(world.get_entity_by_path(&first_path), Some(first));
    assert_eq!(world.get_entity_by_path(&second_path), Some(second));
    assert_eq!(
        world.get_entity_by_path(&EntityPath::parse("Root/Hero").unwrap()),
        None
    );
}

#[test]
fn world_property_reads_compare_normalized_segments_without_entry_vector_allocation() {
    let read_source = include_str!("../world/property_access/read.rs");
    let entries_source = include_str!("../world/property_access/entries.rs");
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");

    assert!(read_source.contains("let target_component = property_path.component();"));
    assert!(read_source.contains("let target_segments = property_path.property_segments();"));
    assert!(read_source
        .contains("self.property_entry_value(entity, target_component, target_segments)"));
    assert!(read_source.contains("return Ok(value);"));
    assert!(read_source
        .contains("if let Some(value) = self.dynamic_component_property(entity, property_path)"));
    assert!(read_source.contains("Err(format!(\n            \"property `{property_path}` is not available on entity {entity}\""));
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

    let source = include_str!("../../core/framework/scene/entity_path.rs");
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
    let path_resolution_source = include_str!("../world/property_access/path_resolution.rs");
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
    let entries_source = include_str!("../world/property_access/entries.rs");

    assert!(
        entries_source.contains("Vec::with_capacity(self.property_entry_capacity_hint(entity))")
    );
    assert!(entries_source.contains("push_entry!("));
    assert!(entries_source.contains("\"Hierarchy.parent\""));
    assert!(entries_source.contains("ScenePropertyValue::Entity(self.parent_of(entity))"));
    assert!(entries_source.contains("fn property_entry_capacity_hint(&self, entity: EntityId)"));
    assert!(entries_source.contains("capacity += 10 + mesh.morph_weights.len();"));
    assert!(entries_source.contains("capacity += 14;"));
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
    let entries_source = include_str!("../world/property_access/entries.rs");
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

#[test]
fn world_property_writes_use_direct_optional_state_branches() {
    let write_source = include_str!("../world/property_access/write.rs");

    assert!(write_source.contains("let next = if next.is_empty() {"));
    assert!(write_source.contains("None"));
    assert!(write_source.contains("} else {"));
    assert!(write_source.contains("Some(next)"));
    assert!(write_source.contains("if let Some(material) = collider.material.as_ref()"));
    assert!(write_source.contains("if material.id() == next"));
    assert!(write_source.contains("return Ok(false);"));
    assert!(!write_source.contains("(!next.is_empty()).then_some(next)"));
    assert!(!write_source.contains(".is_some_and(|handle| handle.id() == next)"));
}

#[test]
fn world_property_writes_pre_size_normalized_segment_vector() {
    let write_source = include_str!("../world/property_access/write.rs");
    let set_property_source = write_source
        .split("fn set_property_impl(")
        .nth(1)
        .and_then(|text| text.split("match component.as_str()").next())
        .expect("read set_property_impl setup");

    assert!(set_property_source.contains("let raw_segments = property_path.property_segments();"));
    assert!(set_property_source.contains("Vec::with_capacity(raw_segments.len())"));
    assert!(set_property_source.contains("for segment in raw_segments"));
    assert!(set_property_source.contains("segments.push(normalized_identifier(segment));"));
    assert!(!set_property_source.contains(".map(|segment| normalized_identifier(segment))"));
    assert!(!set_property_source.contains(".collect::<Vec<_>>()"));
}

#[test]
fn world_collider_shape_kind_write_matches_normalized_values_without_allocation() {
    let write_source = include_str!("../world/property_access/write.rs");
    let shape_kind_source = write_source
        .split("(shape, \"kind\") => {")
        .nth(1)
        .and_then(|text| text.split("if *shape == replacement").next())
        .expect("read collider shape kind write branch");

    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"box\")"));
    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"sphere\")"));
    assert!(shape_kind_source.contains("normalized_identifier_matches(&next_kind, \"capsule\")"));
    assert!(!shape_kind_source.contains("normalized_identifier(&next_kind).as_str()"));
}

#[test]
fn world_property_write_segment_expectation_uses_direct_candidate_loop() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let expect_segment_source = value_conversion_source
        .split("pub(super) fn expect_segment(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn unknown_property_error").next())
        .expect("read expect_segment helper body");

    assert!(expect_segment_source.contains("for candidate in expected"));
    assert!(expect_segment_source.contains("if *candidate == actual"));
    assert!(expect_segment_source.contains("return Ok(());"));
    assert!(!expect_segment_source.contains("expected.iter().any("));
}

#[test]
fn world_transform_rotation_validation_sums_quaternion_length_directly() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let validate_quat_source = value_conversion_source
        .split("pub(super) fn validate_quat_array(")
        .nth(1)
        .and_then(|text| text.split("fn validate_finite_scalar").next())
        .expect("read validate_quat_array helper body");

    assert!(validate_quat_source.contains("let mut length_squared = 0.0;"));
    assert!(validate_quat_source.contains("for component in value"));
    assert!(validate_quat_source.contains("length_squared += component * component;"));
    assert!(validate_quat_source.contains("if length_squared <= Real::EPSILON"));
    assert!(!validate_quat_source.contains(".iter()"));
    assert!(!validate_quat_source.contains(".sum::<Real>()"));
}

#[test]
fn world_property_numeric_array_validation_uses_direct_finite_loop() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let validate_finite_array_source = value_conversion_source
        .split("fn validate_finite_array(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_resource_id").next())
        .expect("read validate_finite_array helper body");

    assert!(validate_finite_array_source.contains("for component in value"));
    assert!(validate_finite_array_source.contains("if !component.is_finite()"));
    assert!(validate_finite_array_source.contains("return Err(format!("));
    assert!(validate_finite_array_source.contains("Ok(())"));
    assert!(!validate_finite_array_source.contains(".iter().all("));
}

#[test]
fn world_property_enum_parsers_match_normalized_values_without_allocation() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let parse_mobility_source = value_conversion_source
        .split("pub(super) fn parse_mobility")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_rigid_body_type").next())
        .expect("read parse_mobility helper body");
    let parse_rigid_body_source = value_conversion_source
        .split("pub(super) fn parse_rigid_body_type")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_joint_kind").next())
        .expect("read parse_rigid_body_type helper body");
    let parse_joint_source = value_conversion_source
        .split("pub(super) fn parse_joint_kind")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn parse_combine_rule").next())
        .expect("read parse_joint_kind helper body");
    let parse_combine_source = value_conversion_source
        .split("pub(super) fn parse_combine_rule")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn combine_rule_label").next())
        .expect("read parse_combine_rule helper body");

    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"dynamic\")"));
    assert!(parse_mobility_source.contains("normalized_identifier_matches(value, \"static\")"));
    assert!(parse_rigid_body_source.contains("normalized_identifier_matches(value, \"kinematic\")"));
    assert!(parse_joint_source.contains("normalized_identifier_matches(value, \"generic6dof\")"));
    assert!(parse_joint_source.contains("normalized_identifier_matches(value, \"sixdof\")"));
    assert!(parse_combine_source.contains("normalized_identifier_matches(value, \"multiply\")"));
    assert!(!parse_mobility_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_rigid_body_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_joint_source.contains("normalized_identifier(value).as_str()"));
    assert!(!parse_combine_source.contains("normalized_identifier(value).as_str()"));
}

#[test]
fn world_property_write_normalizer_pushes_identifier_characters_directly() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let normalized_identifier_source = value_conversion_source
        .split("pub(super) fn normalized_identifier(value: &str) -> String")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn normalized_identifier_matches")
                .next()
        })
        .expect("read normalized_identifier helper body");

    assert!(normalized_identifier_source.contains("String::with_capacity(value.len())"));
    assert!(normalized_identifier_source.contains("for character in value.chars()"));
    assert!(normalized_identifier_source.contains("if character.is_ascii_alphanumeric()"));
    assert!(
        normalized_identifier_source.contains("normalized.push(character.to_ascii_lowercase());")
    );
    assert!(!normalized_identifier_source.contains(".filter(|ch|"));
    assert!(!normalized_identifier_source.contains(".map(|ch|"));
    assert!(!normalized_identifier_source.contains(".collect()"));
}

#[test]
fn world_property_value_conversion_errors_use_direct_result_branches() {
    let value_conversion_source = include_str!("../world/property_access/value_conversion.rs");
    let expect_i32_source = value_conversion_source
        .split("pub(super) fn expect_i32")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn expect_vec3").next())
        .expect("read expect_i32 body");
    let resource_id_source = value_conversion_source
        .split("pub(super) fn expect_resource_id")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn expect_animation_parameter")
                .next()
        })
        .expect("read expect_resource_id body");

    assert!(expect_i32_source
        .contains("ScenePropertyValue::Integer(value) => match i32::try_from(value)"));
    assert!(expect_i32_source
        .contains("ScenePropertyValue::Unsigned(value) => match i32::try_from(value)"));
    assert!(expect_i32_source.contains("Ok(value) => Ok(value)"));
    assert!(expect_i32_source
        .contains("Err(_) => Err(format!(\"property `{property_path}` expected i32 integer\"))"));
    assert!(resource_id_source.contains("match ResourceId::from_str(&value)"));
    assert!(resource_id_source.contains("Ok(resource_id) => Ok(resource_id)"));
    assert!(resource_id_source
        .contains("\"property `{property_path}` has invalid resource id: {error}\""));
    assert!(!expect_i32_source.contains(".map_err("));
    assert!(!resource_id_source.contains(".map_err("));
}

#[test]
fn world_rejects_zero_length_transform_rotation_property_writes() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    let rotation_path = ComponentPropertyPath::parse("Transform.rotation").unwrap();
    let rotation_w_path = ComponentPropertyPath::parse("Transform.rotation.w").unwrap();

    let error = world
        .set_property(
            hero,
            &rotation_path,
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 0.0]),
        )
        .unwrap_err();
    assert!(error.contains("zero-length"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );

    let error = world
        .set_property(hero, &rotation_w_path, ScenePropertyValue::Scalar(0.0))
        .unwrap_err();
    assert!(error.contains("zero-length"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );
}

#[test]
fn world_rejects_non_finite_transform_property_writes() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let translation_x_path = ComponentPropertyPath::parse("Transform.translation.x").unwrap();
    let scale_path = ComponentPropertyPath::parse("Transform.scale").unwrap();

    let error = world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([f32::NAN, 1.0, 2.0]),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &translation_x_path,
            ScenePropertyValue::Scalar(f32::INFINITY),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &scale_path,
            ScenePropertyValue::Vec3([1.0, f32::NEG_INFINITY, 1.0]),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(world.find_node(hero).unwrap().transform.scale, Vec3::ONE);
}
