use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Quat, Vec3};
use crate::scene::components::ColliderShape;
use crate::scene::EntityId;

use super::super::World;
use super::value_conversion::{
    axis_index, expect_animation_parameter, expect_bool, expect_enum, expect_quat,
    expect_resource_id, expect_scalar, expect_segment, expect_segment_count, expect_string,
    expect_u32, expect_vec3, expect_vec4, missing_component_error, normalized_identifier,
    parse_combine_rule, parse_joint_kind, parse_mobility, parse_rigid_body_type,
    property_type_error, quat_axis_index, set_animation_player_like_property,
    unknown_property_error,
};

impl World {
    pub fn set_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> Result<bool, String> {
        self.set_property_impl(entity, property_path, value)
    }

    fn set_property_impl(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update missing entity {entity}"));
        }

        let component = normalized_identifier(property_path.component());
        let segments = property_path
            .property_segments()
            .iter()
            .map(|segment| normalized_identifier(segment))
            .collect::<Vec<_>>();

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
            "camera" => {
                let Some(camera) = self.cameras.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "fovyradians" => {
                        let scalar = expect_scalar(value, property_path)?;
                        if camera.fov_y_radians == scalar {
                            return Ok(false);
                        }
                        camera.fov_y_radians = scalar;
                    }
                    [field] if field == "znear" => {
                        let scalar = expect_scalar(value, property_path)?;
                        if camera.z_near == scalar {
                            return Ok(false);
                        }
                        camera.z_near = scalar;
                    }
                    [field] if field == "zfar" => {
                        let scalar = expect_scalar(value, property_path)?;
                        if camera.z_far == scalar {
                            return Ok(false);
                        }
                        camera.z_far = scalar;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "meshrenderer" | "mesh" => {
                let Some(mesh) = self.mesh_renderers.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "model" => {
                        let resource = expect_resource_id(value, property_path)?;
                        if mesh.model.id() == resource {
                            return Ok(false);
                        }
                        mesh.model = crate::core::resource::ResourceHandle::new(resource);
                    }
                    [field] if field == "material" => {
                        let resource = expect_resource_id(value, property_path)?;
                        if mesh.material.id() == resource {
                            return Ok(false);
                        }
                        mesh.material = crate::core::resource::ResourceHandle::new(resource);
                    }
                    [field] if field == "tint" => {
                        let next = expect_vec4(value, property_path)?;
                        if mesh.tint == next {
                            return Ok(false);
                        }
                        mesh.tint = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "directionallight" | "light" => {
                let Some(light) = self.directional_lights.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "direction" => {
                        let next = expect_vec3(value, property_path)?;
                        if light.direction == next {
                            return Ok(false);
                        }
                        light.direction = next;
                    }
                    [field] if field == "color" => {
                        let next = expect_vec3(value, property_path)?;
                        if light.color == next {
                            return Ok(false);
                        }
                        light.color = next;
                    }
                    [field] if field == "intensity" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.intensity == next {
                            return Ok(false);
                        }
                        light.intensity = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "pointlight" => {
                let Some(light) = self.point_lights.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "color" => {
                        let next = expect_vec3(value, property_path)?;
                        if light.color == next {
                            return Ok(false);
                        }
                        light.color = next;
                    }
                    [field] if field == "intensity" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.intensity == next {
                            return Ok(false);
                        }
                        light.intensity = next;
                    }
                    [field] if field == "range" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.range == next {
                            return Ok(false);
                        }
                        light.range = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "spotlight" => {
                let Some(light) = self.spot_lights.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "direction" => {
                        let next = expect_vec3(value, property_path)?;
                        if light.direction == next {
                            return Ok(false);
                        }
                        light.direction = next;
                    }
                    [field] if field == "color" => {
                        let next = expect_vec3(value, property_path)?;
                        if light.color == next {
                            return Ok(false);
                        }
                        light.color = next;
                    }
                    [field] if field == "intensity" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.intensity == next {
                            return Ok(false);
                        }
                        light.intensity = next;
                    }
                    [field] if field == "range" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.range == next {
                            return Ok(false);
                        }
                        light.range = next;
                    }
                    [field] if field == "innerangleradians" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.inner_angle_radians == next {
                            return Ok(false);
                        }
                        light.inner_angle_radians = next;
                    }
                    [field] if field == "outerangleradians" => {
                        let next = expect_scalar(value, property_path)?;
                        if light.outer_angle_radians == next {
                            return Ok(false);
                        }
                        light.outer_angle_radians = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "rigidbody" => self.set_rigid_body_property(entity, &segments, value, property_path),
            "collider" => self.set_collider_property(entity, &segments, value, property_path),
            "joint" => self.set_joint_property(entity, &segments, value, property_path),
            "animationskeleton" => {
                let Some(skeleton) = self.animation_skeletons.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                match segments.as_slice() {
                    [field] if field == "skeleton" => {
                        let next = expect_resource_id(value, property_path)?;
                        if skeleton.skeleton.id() == next {
                            return Ok(false);
                        }
                        skeleton.skeleton = crate::core::resource::ResourceHandle::new(next);
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.refresh_node_cache();
                Ok(true)
            }
            "animationplayer" => {
                let Some(player) = self.animation_players.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                let changed = set_animation_player_like_property(
                    &segments,
                    value,
                    property_path,
                    &mut player.clip,
                    &mut player.playback_speed,
                    &mut player.time_seconds,
                    Some(&mut player.weight),
                    &mut player.looping,
                    &mut player.playing,
                )?;
                if changed {
                    self.refresh_node_cache();
                }
                Ok(changed)
            }
            "animationsequenceplayer" => {
                let Some(player) = self.animation_sequence_players.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                let changed = set_animation_player_like_property(
                    &segments,
                    value,
                    property_path,
                    &mut player.sequence,
                    &mut player.playback_speed,
                    &mut player.time_seconds,
                    None,
                    &mut player.looping,
                    &mut player.playing,
                )?;
                if changed {
                    self.refresh_node_cache();
                }
                Ok(changed)
            }
            "animationgraphplayer" => {
                let Some(player) = self.animation_graph_players.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                let changed = match segments.as_slice() {
                    [field] if field == "graph" => {
                        let next = expect_resource_id(value, property_path)?;
                        if player.graph.id() == next {
                            false
                        } else {
                            player.graph = crate::core::resource::ResourceHandle::new(next);
                            true
                        }
                    }
                    [field] if field == "playing" => {
                        let next = expect_bool(value, property_path)?;
                        if player.playing == next {
                            false
                        } else {
                            player.playing = next;
                            true
                        }
                    }
                    [parameters, key] if parameters == "parameters" => {
                        let next = expect_animation_parameter(value, property_path)?;
                        if player.parameters.get(key) == Some(&next) {
                            false
                        } else {
                            player.parameters.insert(key.clone(), next);
                            true
                        }
                    }
                    _ => return unknown_property_error(property_path),
                };
                if changed {
                    self.refresh_node_cache();
                }
                Ok(changed)
            }
            "animationstatemachineplayer" => {
                let Some(player) = self.animation_state_machine_players.get_mut(&entity) else {
                    return missing_component_error(entity, property_path);
                };
                let changed = match segments.as_slice() {
                    [field] if field == "statemachine" => {
                        let next = expect_resource_id(value, property_path)?;
                        if player.state_machine.id() == next {
                            false
                        } else {
                            player.state_machine = crate::core::resource::ResourceHandle::new(next);
                            true
                        }
                    }
                    [field] if field == "playing" => {
                        let next = expect_bool(value, property_path)?;
                        if player.playing == next {
                            false
                        } else {
                            player.playing = next;
                            true
                        }
                    }
                    [field] if field == "activestate" => {
                        let next = expect_string(value, property_path)?;
                        let next = (!next.is_empty()).then_some(next);
                        if player.active_state == next {
                            false
                        } else {
                            player.active_state = next;
                            true
                        }
                    }
                    [parameters, key] if parameters == "parameters" => {
                        let next = expect_animation_parameter(value, property_path)?;
                        if player.parameters.get(key) == Some(&next) {
                            false
                        } else {
                            player.parameters.insert(key.clone(), next);
                            true
                        }
                    }
                    _ => return unknown_property_error(property_path),
                };
                if changed {
                    self.refresh_node_cache();
                }
                Ok(changed)
            }
            _ => unknown_property_error(property_path),
        }
    }

    fn set_transform_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> Result<bool, String> {
        let Some(current) = self.local_transforms.get(&entity).copied() else {
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

    fn set_rigid_body_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> Result<bool, String> {
        let Some(rigid_body) = self.rigid_bodies.get_mut(&entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "kind" => {
                let kind = expect_enum(value, property_path)?;
                let next = parse_rigid_body_type(&kind)?;
                if rigid_body.body_type == next {
                    return Ok(false);
                }
                rigid_body.body_type = next;
            }
            [field] if field == "mass" => {
                let next = expect_scalar(value, property_path)?;
                if rigid_body.mass == next {
                    return Ok(false);
                }
                rigid_body.mass = next;
            }
            [field] if field == "lineardamping" => {
                let next = expect_scalar(value, property_path)?;
                if rigid_body.linear_damping == next {
                    return Ok(false);
                }
                rigid_body.linear_damping = next;
            }
            [field] if field == "angulardamping" => {
                let next = expect_scalar(value, property_path)?;
                if rigid_body.angular_damping == next {
                    return Ok(false);
                }
                rigid_body.angular_damping = next;
            }
            [field] if field == "gravityscale" => {
                let next = expect_scalar(value, property_path)?;
                if rigid_body.gravity_scale == next {
                    return Ok(false);
                }
                rigid_body.gravity_scale = next;
            }
            [field] if field == "cansleep" => {
                let next = expect_bool(value, property_path)?;
                if rigid_body.can_sleep == next {
                    return Ok(false);
                }
                rigid_body.can_sleep = next;
            }
            [field, axis] if field == "locktranslation" => {
                let axis = axis_index(axis, property_path)?;
                let next = expect_bool(value, property_path)?;
                if rigid_body.lock_translation[axis] == next {
                    return Ok(false);
                }
                rigid_body.lock_translation[axis] = next;
            }
            [field, axis] if field == "lockrotation" => {
                let axis = axis_index(axis, property_path)?;
                let next = expect_bool(value, property_path)?;
                if rigid_body.lock_rotation[axis] == next {
                    return Ok(false);
                }
                rigid_body.lock_rotation[axis] = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.refresh_node_cache();
        Ok(true)
    }

    fn set_collider_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> Result<bool, String> {
        let Some(collider) = self.colliders.get_mut(&entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "sensor" => {
                let next = expect_bool(value, property_path)?;
                if collider.sensor == next {
                    return Ok(false);
                }
                collider.sensor = next;
            }
            [field] if field == "layer" => {
                let next = expect_u32(value, property_path)?;
                if collider.layer == next {
                    return Ok(false);
                }
                collider.layer = next;
            }
            [field] if field == "collisiongroup" => {
                let next = expect_u32(value, property_path)?;
                if collider.collision_group == next {
                    return Ok(false);
                }
                collider.collision_group = next;
            }
            [field] if field == "collisionmask" => {
                let next = expect_u32(value, property_path)?;
                if collider.collision_mask == next {
                    return Ok(false);
                }
                collider.collision_mask = next;
            }
            [field] if field == "material" => {
                let next = expect_resource_id(value, property_path)?;
                if collider
                    .material
                    .as_ref()
                    .is_some_and(|handle| handle.id() == next)
                {
                    return Ok(false);
                }
                collider.material = Some(crate::core::resource::ResourceHandle::new(next));
            }
            [field, subfield] if field == "materialoverride" => {
                let material_override = collider
                    .material_override
                    .get_or_insert_with(Default::default);
                match subfield.as_str() {
                    "staticfriction" => {
                        let next = expect_scalar(value, property_path)?;
                        if material_override.static_friction == next {
                            return Ok(false);
                        }
                        material_override.static_friction = next;
                    }
                    "dynamicfriction" => {
                        let next = expect_scalar(value, property_path)?;
                        if material_override.dynamic_friction == next {
                            return Ok(false);
                        }
                        material_override.dynamic_friction = next;
                    }
                    "restitution" => {
                        let next = expect_scalar(value, property_path)?;
                        if material_override.restitution == next {
                            return Ok(false);
                        }
                        material_override.restitution = next;
                    }
                    "frictioncombine" => {
                        let next = parse_combine_rule(&expect_enum(value, property_path)?)?;
                        if material_override.friction_combine == next {
                            return Ok(false);
                        }
                        material_override.friction_combine = next;
                    }
                    "restitutioncombine" => {
                        let next = parse_combine_rule(&expect_enum(value, property_path)?)?;
                        if material_override.restitution_combine == next {
                            return Ok(false);
                        }
                        material_override.restitution_combine = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
            }
            [field, transform_field] if field == "localtransform" => {
                let mut next = collider.local_transform;
                match transform_field.as_str() {
                    "translation" => next.translation = expect_vec3(value, property_path)?,
                    "rotation" => next.rotation = expect_quat(value, property_path)?,
                    "scale" => next.scale = expect_vec3(value, property_path)?,
                    _ => return unknown_property_error(property_path),
                }
                if collider.local_transform == next {
                    return Ok(false);
                }
                collider.local_transform = next;
            }
            [field, shape_field] if field == "shape" => {
                match (&mut collider.shape, shape_field.as_str()) {
                    (shape, "kind") => {
                        let next_kind = expect_enum(value, property_path)?;
                        let replacement = match normalized_identifier(&next_kind).as_str() {
                            "box" => ColliderShape::Box {
                                half_extents: Vec3::splat(0.5),
                            },
                            "sphere" => ColliderShape::Sphere { radius: 0.5 },
                            "capsule" => ColliderShape::Capsule {
                                radius: 0.5,
                                half_height: 0.5,
                            },
                            _ => return Err(format!("unsupported collider shape `{next_kind}`")),
                        };
                        if *shape == replacement {
                            return Ok(false);
                        }
                        *shape = replacement;
                    }
                    (ColliderShape::Box { half_extents }, "halfextents") => {
                        let next = expect_vec3(value, property_path)?;
                        if *half_extents == next {
                            return Ok(false);
                        }
                        *half_extents = next;
                    }
                    (ColliderShape::Sphere { radius }, "radius") => {
                        let next = expect_scalar(value, property_path)?;
                        if *radius == next {
                            return Ok(false);
                        }
                        *radius = next;
                    }
                    (ColliderShape::Capsule { radius, .. }, "radius") => {
                        let next = expect_scalar(value, property_path)?;
                        if *radius == next {
                            return Ok(false);
                        }
                        *radius = next;
                    }
                    (ColliderShape::Capsule { half_height, .. }, "halfheight") => {
                        let next = expect_scalar(value, property_path)?;
                        if *half_height == next {
                            return Ok(false);
                        }
                        *half_height = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
            }
            _ => return unknown_property_error(property_path),
        }
        self.refresh_node_cache();
        Ok(true)
    }

    fn set_joint_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> Result<bool, String> {
        let Some(joint) = self.joints.get_mut(&entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "kind" => {
                let next = parse_joint_kind(&expect_enum(value, property_path)?)?;
                if joint.joint_type == next {
                    return Ok(false);
                }
                joint.joint_type = next;
            }
            [field] if field == "connectedentity" => {
                let ScenePropertyValue::Entity(next) = value else {
                    return property_type_error(property_path, "entity reference");
                };
                if joint.connected_entity == next {
                    return Ok(false);
                }
                joint.connected_entity = next;
            }
            [field] if field == "anchor" => {
                let next = expect_vec3(value, property_path)?;
                if joint.anchor == next {
                    return Ok(false);
                }
                joint.anchor = next;
            }
            [field] if field == "axis" => {
                let next = expect_vec3(value, property_path)?;
                if joint.axis == next {
                    return Ok(false);
                }
                joint.axis = next;
            }
            [field, bound] if field == "limits" => {
                let next = expect_scalar(value, property_path)?;
                let limits = joint.limits.get_or_insert([0.0, 0.0]);
                let index = match bound.as_str() {
                    "min" => 0,
                    "max" => 1,
                    _ => return unknown_property_error(property_path),
                };
                if limits[index] == next {
                    return Ok(false);
                }
                limits[index] = next;
            }
            [field] if field == "collideconnected" => {
                let next = expect_bool(value, property_path)?;
                if joint.collide_connected == next {
                    return Ok(false);
                }
                joint.collide_connected = next;
            }
            _ => return unknown_property_error(property_path),
        }
        self.refresh_node_cache();
        Ok(true)
    }
}
