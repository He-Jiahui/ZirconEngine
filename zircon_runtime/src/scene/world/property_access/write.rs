mod animation;
mod camera;
mod lighting;
mod mesh;
mod physics;

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::Quat;
use crate::scene::components::LocalTransform;
use crate::scene::{EntityId, SceneError, SceneResult};

use super::super::World;
use super::value_conversion::{
    axis_index, expect_quat, expect_scalar, expect_segment, expect_segment_count, expect_u32,
    expect_vec3, missing_component_error, normalized_identifier, parse_mobility,
    property_type_error, quat_axis_index, unknown_property_error, validate_quat_array,
};

impl World {
    pub fn set_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let generation = self.world_generation();
        let changed = self.set_property_impl(entity, property_path, value)?;
        if changed {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
            if self.world_generation() == generation {
                // Some established fixed components are edited in place below. They still
                // publish a fresh inspection generation, while their hierarchy rows remain valid.
                self.advance_world_generation();
            }
        }
        Ok(changed)
    }

    fn set_property_impl(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update", entity));
        }

        let component = normalized_identifier(property_path.component());
        let raw_segments = property_path.property_segments();
        let mut segments = Vec::with_capacity(raw_segments.len());
        for segment in raw_segments {
            segments.push(normalized_identifier(segment));
        }

        match component.as_str() {
            "name" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["value"], property_path)?;
                let ScenePropertyValue::String(name) = value else {
                    return property_type_error(property_path, "string");
                };
                self.rename_node(entity, name)
            }
            "hierarchy" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["parent"], property_path)?;
                let ScenePropertyValue::Entity(parent) = value else {
                    return property_type_error(property_path, "entity reference");
                };
                self.set_parent_checked(entity, parent)
            }
            "transform" => self.set_transform_property(entity, &segments, value, property_path),
            "active" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["enabled"], property_path)?;
                let ScenePropertyValue::Bool(active) = value else {
                    return property_type_error(property_path, "bool");
                };
                self.set_active_self(entity, active)
            }
            "renderlayer" | "renderlayermask" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["mask"], property_path)?;
                let mask = expect_u32(value, property_path)?;
                self.set_render_layer_mask(entity, mask)
            }
            "mobility" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["kind"], property_path)?;
                let ScenePropertyValue::Enum(kind) = value else {
                    return property_type_error(property_path, "enum");
                };
                self.set_mobility(entity, parse_mobility(&kind)?)
            }
            "camera" => self.set_camera_property(entity, &segments, value, property_path),
            "meshrenderer" | "mesh" => {
                self.set_mesh_renderer_property(entity, &segments, value, property_path)
            }
            "ambientlight" => {
                self.set_ambient_light_property(entity, &segments, value, property_path)
            }
            "directionallight" | "light" => {
                self.set_directional_light_property(entity, &segments, value, property_path)
            }
            "pointlight" => self.set_point_light_property(entity, &segments, value, property_path),
            "rectlight" => self.set_rect_light_property(entity, &segments, value, property_path),
            "spotlight" => self.set_spot_light_property(entity, &segments, value, property_path),
            "rigidbody" => self.set_rigid_body_property(entity, &segments, value, property_path),
            "collider" => self.set_collider_property(entity, &segments, value, property_path),
            "joint" => self.set_joint_property(entity, &segments, value, property_path),
            "animationskeleton" => {
                self.set_animation_skeleton_property(entity, &segments, value, property_path)
            }
            "animationplayer" => {
                self.set_animation_player_property(entity, &segments, value, property_path)
            }
            "animationsequenceplayer" => {
                self.set_animation_sequence_player_property(entity, &segments, value, property_path)
            }
            "animationgraphplayer" => {
                self.set_animation_graph_player_property(entity, &segments, value, property_path)
            }
            "animationstatemachineplayer" => self.set_animation_state_machine_player_property(
                entity,
                &segments,
                value,
                property_path,
            ),
            _ => self.set_dynamic_component_property(entity, property_path, value),
        }
    }

    fn set_transform_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(current) = self.get::<LocalTransform>(entity).copied() else {
            return missing_component_error(entity, property_path);
        };
        let mut next = current.transform;
        match segments {
            [field] if field == "translation" => {
                next.translation = expect_vec3(value, property_path)?
            }
            [field] if field == "rotation" => next.rotation = expect_quat(value, property_path)?,
            [field] if field == "scale" => next.scale = expect_vec3(value, property_path)?,
            [field, axis] if field == "translation" => {
                let axis = axis_index(axis, property_path)?;
                next.translation[axis] = expect_scalar(value, property_path)?;
            }
            [field, axis] if field == "rotation" => {
                let axis = quat_axis_index(axis, property_path)?;
                let mut rotation = next.rotation.to_array();
                rotation[axis] = expect_scalar(value, property_path)?;
                validate_quat_array(rotation, property_path)?;
                next.rotation = Quat::from_array(rotation);
            }
            [field, axis] if field == "scale" => {
                let axis = axis_index(axis, property_path)?;
                next.scale[axis] = expect_scalar(value, property_path)?;
            }
            _ => return unknown_property_error(property_path),
        }
        self.update_transform(entity, next)
    }
}
