use super::*;

#[test]
fn world_property_uses_direct_static_dispatch_without_inspector_enumeration() {
    let read_source = include_str!("../../world/property_access/read.rs");
    let entries_source = include_str!("../../world/property_access/entries.rs");
    let physics_source = include_str!("../../world/property_access/entries/physics.rs");
    let collider_shape_source =
        include_str!("../../world/property_access/entries/collider_shape.rs");

    assert!(read_source.contains("self.static_property_value("));
    assert!(read_source.contains("fn static_property_value("));
    assert!(read_source.contains("macro_rules! direct_property_field"));
    assert!(read_source.contains("self.mesh_renderer_property_value(entity, segments)"));
    assert!(read_source.contains("self.physics_property_value(entity, component, segments)"));
    assert!(read_source.contains("return Ok(value);"));
    assert!(
        read_source.contains(
            "if let Some(value) = self.dynamic_component_property(entity, property_path)"
        )
    );
    assert!(read_source.contains(") -> SceneResult<ScenePropertyValue>"));
    assert!(read_source.contains("SceneError::PropertyUnavailable"));
    assert!(read_source.contains("property_path: property_path.to_string()"));
    assert!(!read_source.contains("self.property_entry_value("));
    assert!(!read_source.contains("self.property_entries(entity)"));
    assert!(!entries_source.contains("pub(super) fn property_entry_value("));
    assert!(entries_source.contains("fn visit_property_entries<"));
    assert!(entries_source.contains("macro_rules! push_entry"));
    assert!(entries_source.contains("let mut build_value = || $value;"));
    assert!(entries_source.contains("if !visitor($path, &mut build_value, $animatable)"));
    assert!(entries_source.contains("if include_dynamic {"));
    assert!(!entries_source.contains("property_path_literal_matches_normalized"));
    assert!(physics_source.contains("pub(super) fn physics_property_value("));
    assert!(physics_source.contains("collider_shape_property_value(&collider.shape, remaining)"));
    assert!(collider_shape_source.contains("pub(super) fn collider_shape_property_value("));
    assert!(!read_source.contains("let entries = self.property_entries(entity);"));
    assert!(!read_source.contains("entries\n            .into_iter()"));
    assert!(!read_source.contains("fn property_path_matches_normalized("));
    assert!(!read_source.contains("fn property_segments_match_normalized("));
    assert!(
        !entries_source
            .contains("let mut push = |path: &str, value: ScenePropertyValue, animatable: bool|")
    );
    assert!(!read_source.contains("use super::value_conversion::normalized_identifier;"));
    assert!(!read_source.contains("let target_component = normalized_identifier("));
    assert!(
        !read_source
            .contains(".or_else(|| self.dynamic_component_property(entity, property_path))")
    );
    assert!(!read_source.contains(".ok_or_else(||"));
    assert!(!read_source.contains(
        ".property_segments()\n                        .iter()\n                        .map(|segment| normalized_identifier(segment))\n                        .collect::<Vec<_>>()"
    ));
    assert!(!read_source.contains(".map(|segment| normalized_identifier(segment))"));
    assert!(!read_source.contains(".collect::<Vec<_>>()"));
    assert!(
        !read_source
            .contains("normalized_identifier(property_path.component()) == target_component")
    );
    assert!(
        !read_source.contains("normalized_identifier(&segments[index]) != target_segments[index]")
    );
    assert!(!read_source.contains(".zip(target_segments)"));
    assert!(
        !read_source
            .contains(".all(|(segment, target)| normalized_identifier(segment) == *target)")
    );
}

#[test]
fn world_property_materializes_only_the_requested_entry() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(entity, "Target").unwrap();
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        )
        .unwrap();

    world.reset_compiled_scene_property_access_stats();
    let translation = ComponentPropertyPath::parse("Transform.translation").unwrap();
    assert_eq!(
        world.property(entity, &translation).unwrap(),
        ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
    );

    let stats = world.compiled_scene_property_access_stats();
    assert_eq!(stats.property_entry_visits, 1);
    assert_eq!(stats.path_lookup_requests, 0);
    assert_eq!(stats.canonicalization_bytes, 0);
}

#[test]
fn world_property_rejects_removed_entity_before_static_dispatch() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.remove_entity(entity).unwrap();

    let hierarchy_parent = ComponentPropertyPath::parse("Hierarchy.parent").unwrap();
    let error = world.property(entity, &hierarchy_parent).unwrap_err();
    assert!(matches!(
        error,
        crate::scene::SceneError::PropertyUnavailable {
            entity: rejected,
            ..
        } if rejected == entity
    ));
}

#[test]
fn world_property_direct_dispatch_preserves_nested_shape_and_sequence_fields() {
    use crate::core::resource::{AnimationSequenceMarker, ResourceHandle, ResourceId};
    use crate::scene::components::{
        AnimationSequencePlayerComponent, ColliderComponent, ColliderShape,
    };

    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Compound {
                    children: vec![(
                        Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
                        Box::new(ColliderShape::Sphere { radius: 0.75 }),
                    )],
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();

    let sequence = ResourceHandle::<AnimationSequenceMarker>::new(ResourceId::from_stable_label(
        "res://animation/direct_dispatch.sequence.zranim",
    ));
    let expected_sequence = sequence.id().to_string();
    world
        .set_animation_sequence_player(
            entity,
            Some(AnimationSequencePlayerComponent {
                sequence,
                playback_speed: 1.0,
                time_seconds: 0.0,
                looping: false,
                playing: true,
            }),
        )
        .unwrap();

    let nested_radius =
        ComponentPropertyPath::parse("Collider.shape.children.0.shape.radius").unwrap();
    let sequence_path = ComponentPropertyPath::parse("AnimationSequencePlayer.sequence").unwrap();
    assert_eq!(
        world.property(entity, &nested_radius).unwrap(),
        ScenePropertyValue::Scalar(0.75)
    );
    assert_eq!(
        world.property(entity, &sequence_path).unwrap(),
        ScenePropertyValue::Resource(expected_sequence)
    );
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

    assert!(
        path_resolution_source
            .contains("pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId>")
    );
    assert!(!path_resolution_source.contains(&format!("pub fn {old_entity_path_lookup}")));
    assert!(path_resolution_source.contains("let target_segments = path.segments();"));
    assert!(path_resolution_source.contains("let mut entity_index = 0;"));
    assert!(path_resolution_source.contains("while entity_index < self.entities.len()"));
    assert!(path_resolution_source.contains("let entity = self.entities[entity_index];"));
    assert!(
        path_resolution_source
            .contains("if self.entity_matches_path_segments(entity, target_segments)")
    );
    assert!(path_resolution_source.contains("return Some(entity);"));
    assert!(path_resolution_source.contains("entity_index += 1;"));
    assert!(path_resolution_source.contains("\n        None\n"));
    assert!(
        path_resolution_source
            .contains("Vec::with_capacity(self.entity_path_segment_capacity(entity))")
    );
    assert!(
        path_resolution_source
            .contains("fn entity_path_segment_capacity(&self, entity: EntityId) -> usize")
    );
    assert!(path_resolution_source.contains("capacity += 1;"));
    assert!(path_resolution_source.contains(
        "fn entity_matches_path_segments(&self, entity: EntityId, target_segments: &[String])"
    ));
    assert!(path_resolution_source.contains("let mut segment_index = target_segments.len();"));
    assert!(
        path_resolution_source.contains("if segment_index == 0 {\n                return false;")
    );
    assert!(path_resolution_source.contains("segment_index -= 1;"));
    assert!(path_resolution_source.contains(
        "if !self.entity_path_segment_matches(current, &target_segments[segment_index])"
    ));
    assert!(path_resolution_source.contains("segment_index == 0"));
    assert!(path_resolution_source.contains("fn entity_path_segment_matches("));
    assert!(path_resolution_source.contains("decimal_entity_id_matches"));
    let entity_matches = path_resolution_source
        .split("fn entity_matches_path_segments(")
        .nth(1)
        .and_then(|text| text.split("fn entity_path_segment_capacity").next())
        .expect("read allocation-free entity path match body");
    assert!(!entity_matches.contains("self.path_segment_for_entity(current)"));
    assert!(!entity_matches.contains("String"));
    assert!(path_resolution_source.contains("fn entity_has_duplicate_path_name("));
    assert!(path_resolution_source.contains("let mut candidate_index = 0;"));
    assert!(path_resolution_source.contains("while candidate_index < self.entities.len()"));
    assert!(path_resolution_source.contains("let candidate = self.entities[candidate_index];"));
    assert!(path_resolution_source.contains("candidate_index += 1;"));
    assert!(
        path_resolution_source
            .contains("if candidate == entity || self.parent_of(candidate) != parent")
    );
    assert!(
        path_resolution_source
            .contains("let Some(candidate_name) = self.get::<Name>(candidate) else")
    );
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
    assert!(
        !path_resolution_source
            .contains("self.entities\n            .iter()\n            .copied()")
    );
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
    assert!(
        physics_entries_source
            .contains("if let Some(collider) = self.get::<ColliderComponent>(entity)")
    );
    assert!(
        physics_entries_source
            .contains("capacity += collider_shape_property_entry_capacity(&collider.shape);")
    );
    assert!(collider_shape_entries_source.contains(
        "pub(super) fn collider_shape_property_entry_capacity(shape: &ColliderShape) -> usize"
    ));
    assert!(
        collider_shape_entries_source
            .contains("3 + collider_shape_property_entry_capacity(child_shape.as_ref())")
    );
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
    assert!(
        entries_source.contains(
            "write!(&mut path, \"{index}\").expect(\"writing to a String cannot fail\");"
        )
    );
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
    assert!(
        json_projection_source.contains("return Some(ScenePropertyValue::Scalar(value as _));")
    );
    assert!(!json_projection_source.contains(".map(ScenePropertyValue::Integer)"));
    assert!(!json_projection_source.contains(".or_else(||"));
}

#[test]
fn compiled_scene_property_target_reuses_normalized_component_field_identity() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let canonical_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let equivalent_path = ComponentPropertyPath::parse("transform.translation").unwrap();
    let canonical = world
        .compile_scene_property_target(&entity_path, &canonical_path)
        .unwrap();
    let equivalent = world
        .compile_scene_property_target(&entity_path, &equivalent_path)
        .unwrap();

    assert_eq!(
        canonical.component_field_id(),
        equivalent.component_field_id()
    );
}

#[test]
fn compiled_sequence_apply_keeps_path_resolution_and_string_dispatch_at_compile_boundary() {
    let compiled_binding_facade = include_str!("../../world/compiled_binding/mod.rs");
    let property_writer_facade = include_str!("../../world/compiled_binding/property_path.rs");
    let property_writer_model = include_str!("../../world/compiled_binding/property_path/model.rs");
    let property_writer_compile =
        include_str!("../../world/compiled_binding/property_path/compile.rs");
    let property_writer_write = include_str!("../../world/compiled_binding/property_path/write.rs");
    let transaction_source = include_str!("../../world/transaction.rs");
    let level_system_source = include_str!("../../level_system.rs");
    let animation_apply_source = include_str!("../../../animation/sequence/compiled.rs");
    let apply_source = animation_apply_source
        .split("pub fn apply_compiled_sequence_to_world")
        .nth(1)
        .expect("read compiled sequence frame-apply body");

    assert!(compiled_binding_facade.contains("mod scene_binding_topology;"));
    assert!(compiled_binding_facade.contains("pub use property_path::{"));
    assert!(!compiled_binding_facade.contains("impl "));
    assert!(!compiled_binding_facade.contains(" fn "));
    assert!(property_writer_facade.contains("mod compile;"));
    assert!(property_writer_facade.contains("mod model;"));
    assert!(property_writer_facade.contains("mod read;"));
    assert!(property_writer_facade.contains("mod write;"));
    assert!(property_writer_facade.contains("pub use model::{"));
    assert!(!property_writer_facade.contains("impl "));
    assert!(!property_writer_facade.contains(" fn "));
    assert!(property_writer_model.contains("pub struct PathId"));
    assert!(property_writer_model.contains("pub struct ComponentFieldId"));
    assert!(property_writer_compile.contains("pub fn compile_scene_property_writer("));
    assert!(property_writer_write.contains("pub fn write_compiled_scene_property("));
    assert!(
        property_writer_compile.contains("pub(crate) fn compile_scene_property_writer_for_entity(")
    );
    assert!(property_writer_compile.contains("canonical_entity_path: &EntityPath"));
    assert!(animation_apply_source.contains("world.entity_path(entity)"));
    assert!(animation_apply_source.contains("compile_scene_property_writer_for_entity("));
    assert!(transaction_source.contains("staged.advance_scene_binding_generations_after(self);"));
    assert!(
        transaction_source
            .contains("staged.advance_world_generation_after(self.world_generation());")
    );
    assert!(
        level_system_source.contains("world.advance_scene_binding_generations_after(&current);")
    );
    assert!(
        level_system_source
            .contains("world.advance_world_generation_after(current.world_generation());")
    );
    assert!(!apply_source.contains("get_entity_by_path"));
    assert!(!apply_source.contains("set_property("));
    assert!(!apply_source.contains("AnimationTrackPath::new"));
}
