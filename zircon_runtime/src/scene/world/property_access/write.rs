mod physics;

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::Quat;
use crate::scene::EntityId;

use super::super::World;
use super::value_conversion::{
    axis_index, expect_animation_parameter, expect_bool, expect_i32, expect_quat,
    expect_resource_id, expect_scalar, expect_segment, expect_segment_count, expect_string,
    expect_u32, expect_vec2, expect_vec3, expect_vec4, missing_component_error,
    normalized_identifier, parse_mobility, property_type_error, quat_axis_index,
    set_animation_player_like_property, unknown_property_error, validate_quat_array,
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
                    .map_err(|error| error.to_string())
            }
            "hierarchy" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["parent"], property_path)?;
                let ScenePropertyValue::Entity(parent) = value else {
                    return property_type_error(property_path, "entity reference");
                };
                self.set_parent_checked(entity, parent)
                    .map_err(|error| error.to_string())
            }
            "transform" => self.set_transform_property(entity, &segments, value, property_path),
            "active" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["enabled"], property_path)?;
                let ScenePropertyValue::Bool(active) = value else {
                    return property_type_error(property_path, "bool");
                };
                self.set_active_self(entity, active)
                    .map_err(|error| error.to_string())
            }
            "renderlayer" | "renderlayermask" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["mask"], property_path)?;
                let mask = expect_u32(value, property_path)?;
                self.set_render_layer_mask(entity, mask)
                    .map_err(|error| error.to_string())
            }
            "mobility" => {
                expect_segment_count(&segments, 1, property_path)?;
                expect_segment(&segments[0], &["kind"], property_path)?;
                let ScenePropertyValue::Enum(kind) = value else {
                    return property_type_error(property_path, "enum");
                };
                self.set_mobility(entity, parse_mobility(&kind)?)
                    .map_err(|error| error.to_string())
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
                self.mark_node_cache_dirty();
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
                    [field] if field == "mesh" => {
                        return Err(format!(
                            "property {property_path} is read-only optional mesh resource"
                        ));
                    }
                    [field] if field == "material" => {
                        let resource = expect_resource_id(value, property_path)?;
                        if mesh.material.id() == resource {
                            return Ok(false);
                        }
                        mesh.material = crate::core::resource::ResourceHandle::new(resource);
                    }
                    [field] if field == "renderqueue" => {
                        let next = expect_i32(value, property_path)?;
                        if mesh.render_queue == next {
                            return Ok(false);
                        }
                        mesh.render_queue = next;
                    }
                    [field] if field == "materialqueue" => {
                        let next = expect_i32(value, property_path)?;
                        if mesh.material_queue == next {
                            return Ok(false);
                        }
                        mesh.material_queue = next;
                    }
                    [field] if field == "orderinlayer" => {
                        let next = expect_i32(value, property_path)?;
                        if mesh.order_in_layer == next {
                            return Ok(false);
                        }
                        mesh.order_in_layer = next;
                    }
                    [field] if field == "depthbias" => {
                        let next = expect_scalar(value, property_path)?;
                        if mesh.depth_bias == next {
                            return Ok(false);
                        }
                        mesh.depth_bias = next;
                    }
                    [field] if field == "primitivebindingcount" || field == "primitives" => {
                        return Err(format!(
                            "property {property_path} is read-only mesh primitive binding data"
                        ));
                    }
                    [field] if field == "lodlevelcount" || field == "lods" => {
                        return Err(format!(
                            "property {property_path} is read-only mesh LOD data"
                        ));
                    }
                    [field] if field == "morphweightcount" => {
                        return Err(format!(
                            "property {property_path} is read-only mesh morph weight count"
                        ));
                    }
                    [field] if field == "morphweights" => {
                        return Err(format!(
                            "property {property_path} must address a morph weight index"
                        ));
                    }
                    [field, index] if field == "morphweights" => {
                        let index = index.parse::<usize>().map_err(|_| {
                            format!("property `{property_path}` has an invalid morph weight index")
                        })?;
                        let next = expect_scalar(value, property_path)?;
                        let resized = if mesh.morph_weights.len() <= index {
                            mesh.morph_weights.resize(index + 1, 0.0);
                            true
                        } else {
                            false
                        };
                        if !resized && mesh.morph_weights[index] == next {
                            return Ok(false);
                        }
                        mesh.morph_weights[index] = next;
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
                self.mark_node_cache_dirty();
                Ok(true)
            }
            "ambientlight" => {
                let Some(light) = self.ambient_lights.get_mut(&entity) else {
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
                    [field] if field == "affectslightmappedmeshes" => {
                        let next = expect_bool(value, property_path)?;
                        if light.affects_lightmapped_meshes == next {
                            return Ok(false);
                        }
                        light.affects_lightmapped_meshes = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.mark_node_cache_dirty();
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
                self.mark_node_cache_dirty();
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
                self.mark_node_cache_dirty();
                Ok(true)
            }
            "rectlight" => {
                let Some(light) = self.rect_lights.get_mut(&entity) else {
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
                    [field] if field == "size" => {
                        let next = expect_vec2(value, property_path)?;
                        if light.size == next {
                            return Ok(false);
                        }
                        light.size = next;
                    }
                    _ => return unknown_property_error(property_path),
                }
                self.mark_node_cache_dirty();
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
                self.mark_node_cache_dirty();
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
                self.mark_node_cache_dirty();
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
                    self.mark_node_cache_dirty();
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
                    self.mark_node_cache_dirty();
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
                    self.mark_node_cache_dirty();
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
                        let next = if next.is_empty() { None } else { Some(next) };
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
                    self.mark_node_cache_dirty();
                }
                Ok(changed)
            }
            _ => self
                .set_dynamic_component_property(entity, property_path, value)
                .map_err(|error| error.to_string()),
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
            .map_err(|error| error.to_string())
    }
}
