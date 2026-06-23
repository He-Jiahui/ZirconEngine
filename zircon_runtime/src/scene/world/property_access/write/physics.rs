use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::Vec3;
use crate::scene::components::ColliderShape;
use crate::scene::EntityId;

use super::super::super::World;
use super::super::value_conversion::{
    axis_index, expect_bool, expect_enum, expect_quat, expect_resource_id, expect_scalar,
    expect_u32, expect_vec3, missing_component_error, normalized_identifier_matches,
    parse_combine_rule, parse_joint_kind, parse_rigid_body_type, property_type_error,
    unknown_property_error,
};

impl World {
    pub(super) fn set_rigid_body_property(
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
            [field] if field == "linearvelocity" => {
                let next = expect_vec3(value, property_path)?;
                if rigid_body.linear_velocity == next {
                    return Ok(false);
                }
                rigid_body.linear_velocity = next;
            }
            [field, axis] if field == "linearvelocity" => {
                let axis = axis_index(axis, property_path)?;
                let mut next = rigid_body.linear_velocity;
                next[axis] = expect_scalar(value, property_path)?;
                if rigid_body.linear_velocity == next {
                    return Ok(false);
                }
                rigid_body.linear_velocity = next;
            }
            [field] if field == "angularvelocity" => {
                let next = expect_vec3(value, property_path)?;
                if rigid_body.angular_velocity == next {
                    return Ok(false);
                }
                rigid_body.angular_velocity = next;
            }
            [field, axis] if field == "angularvelocity" => {
                let axis = axis_index(axis, property_path)?;
                let mut next = rigid_body.angular_velocity;
                next[axis] = expect_scalar(value, property_path)?;
                if rigid_body.angular_velocity == next {
                    return Ok(false);
                }
                rigid_body.angular_velocity = next;
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
            _ => {
                return self
                    .set_dynamic_component_property(entity, property_path, value)
                    .map_err(|error| error.to_string());
            }
        }
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_collider_property(
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
                if let Some(material) = collider.material.as_ref() {
                    if material.id() == next {
                        return Ok(false);
                    }
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
                        let replacement = if normalized_identifier_matches(&next_kind, "box") {
                            ColliderShape::Box {
                                half_extents: Vec3::splat(0.5),
                            }
                        } else if normalized_identifier_matches(&next_kind, "sphere") {
                            ColliderShape::Sphere { radius: 0.5 }
                        } else if normalized_identifier_matches(&next_kind, "capsule") {
                            ColliderShape::Capsule {
                                radius: 0.5,
                                half_height: 0.5,
                            }
                        } else {
                            return Err(format!("unsupported collider shape `{next_kind}`"));
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
        self.mark_node_cache_dirty();
        Ok(true)
    }

    pub(super) fn set_joint_property(
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
        self.mark_node_cache_dirty();
        Ok(true)
    }
}
