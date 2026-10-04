use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::MeshRenderer;
use crate::scene::{EntityId, SceneError, SceneResult};

use super::super::super::World;
use super::super::value_conversion::{
    expect_i32, expect_resource_id, expect_scalar, expect_vec4, missing_component_error,
    unknown_property_error,
};

impl World {
    // 网格属性编辑保持组件值和节点缓存一致；材质资源是否就绪留给渲染提取与资源流处理。
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
                // TODO: [CR-R02-runtime_world_property_binding-0005] 待确认 morph 索引是否须受资源目标数或配置上限约束；此处仅解析 usize，现有测试只覆盖小索引，缺少维度契约；下一步核对资源与编辑器调用方，再验证越界拒绝。
                let index =
                    index
                        .parse::<usize>()
                        .map_err(|_| SceneError::InvalidPropertyIndex {
                            property_path: property_path.to_string(),
                            index_kind: "morph weight index",
                        })?;
                let next = expect_scalar(value, property_path)?;
                let resized = if mesh.morph_weights.len() <= index {
                    // BUG: [CR-R02-runtime_world_property_binding-0006] 路径索引为目标平台 usize::MAX 且数值合法时必 panic；开启溢出检查在加一处失败，关闭检查后长度变 0，随后最大索引赋值越界。证据：此扩容与下方赋值的完整路径。
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
