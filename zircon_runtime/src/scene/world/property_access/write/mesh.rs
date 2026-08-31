use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::MeshRenderer;
use crate::scene::{EntityId, SceneError, SceneResult};

use super::super::super::World;
use super::super::value_conversion::{
    expect_i32, expect_resource_id, expect_scalar, expect_vec4, missing_component_error,
    unknown_property_error,
};

impl World {
    pub(super) fn set_mesh_renderer_property(
        &mut self,
        entity: EntityId,
        segments: &[String],
        value: ScenePropertyValue,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<bool> {
        let Some(mesh) = self.get_mut::<MeshRenderer>(entity) else {
            return missing_component_error(entity, property_path);
        };
        match segments {
            [field] if field == "model" => {
                let resource = expect_resource_id(value, property_path)?;
                if mesh.model.id() == resource {
                    return Ok(false);
                }
                mesh.model = crate::core::resource::ResourceHandle::new(resource);
            }
            [field] if field == "mesh" => {
                return Err(SceneError::ReadOnlyProperty {
                    property_path: property_path.to_string(),
                    reason: "optional mesh resource",
                });
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
                return Err(SceneError::ReadOnlyProperty {
                    property_path: property_path.to_string(),
                    reason: "mesh primitive binding data",
                });
            }
            [field] if field == "lodlevelcount" || field == "lods" => {
                return Err(SceneError::ReadOnlyProperty {
                    property_path: property_path.to_string(),
                    reason: "mesh LOD data",
                });
            }
            [field] if field == "morphweightcount" => {
                return Err(SceneError::ReadOnlyProperty {
                    property_path: property_path.to_string(),
                    reason: "mesh morph weight count",
                });
            }
            [field] if field == "morphweights" => {
                return Err(SceneError::InvalidPropertyIndex {
                    property_path: property_path.to_string(),
                    index_kind: "morph weight index",
                });
            }
            [field, index] if field == "morphweights" => {
                let index =
                    index
                        .parse::<usize>()
                        .map_err(|_| SceneError::InvalidPropertyIndex {
                            property_path: property_path.to_string(),
                            index_kind: "morph weight index",
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
}
