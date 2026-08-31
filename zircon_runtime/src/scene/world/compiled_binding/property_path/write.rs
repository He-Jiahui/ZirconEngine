use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::Quat;
use crate::scene::components::{CameraComponent, LocalTransform, MeshRenderer};
use crate::scene::{SceneError, SceneResult, World};

use super::super::compiled_scene_camera_light_fields::CompiledCameraProperty;
use super::model::{
    CompiledMeshRendererProperty, CompiledScenePropertyWriter, CompiledScenePropertyWriterKind,
    CompiledTransformProperty,
};

impl World {
    /// Writes a compiled scene property without normalizing or dispatching a
    /// string property path during steady-state application.
    pub fn write_compiled_scene_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        if !target.is_current_for(self) {
            self.record_compiled_scene_property_stale_target();
            return Err(SceneError::PropertyUnavailable {
                entity: target.entity(),
                property_path: target.property_path.to_string(),
            });
        }
        self.record_compiled_scene_property_writer_access();
        let generation = self.world_generation();
        let changed = match &target.property {
            CompiledScenePropertyWriterKind::Transform(property) => {
                self.write_compiled_transform_property(target, *property, value)
            }
            CompiledScenePropertyWriterKind::MeshRenderer(property) => {
                self.write_compiled_mesh_renderer_property(target, *property, value)
            }
            CompiledScenePropertyWriterKind::AnimationRuntime(property) => self
                .write_compiled_animation_runtime_property(
                    target.entity(),
                    *property,
                    &target.property_path,
                    value,
                ),
            CompiledScenePropertyWriterKind::Camera(property) => {
                self.write_compiled_camera_property(target, *property, value)
            }
            CompiledScenePropertyWriterKind::Light(property) => {
                self.write_compiled_light_property(target, *property, value)
            }
            CompiledScenePropertyWriterKind::Dynamic(property) => self
                .set_compiled_dynamic_component_property(
                    target.entity(),
                    property.component_id(),
                    property.property(),
                    &target.property_path,
                    value,
                ),
        }?;
        if changed {
            self.inspection_artifact_cache
                .mark_fields_dirty(target.entity());
            if self.world_generation() == generation {
                self.advance_world_generation();
            }
        }
        Ok(changed)
    }

    fn write_compiled_transform_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        property: CompiledTransformProperty,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let Some(local) = self.get::<LocalTransform>(target.entity()).copied() else {
            return Err(SceneError::MissingRequiredComponent {
                operation: "write compiled transform property",
                entity: target.entity(),
                component: "LocalTransform",
            });
        };

        let mut transform = local.transform;
        match property {
            CompiledTransformProperty::Translation => {
                transform.translation =
                    Self::compiled_property_expect_vec3(value, &target.property_path)?;
            }
            CompiledTransformProperty::TranslationAxis(axis) => {
                transform.translation[axis] =
                    Self::compiled_property_expect_scalar(value, &target.property_path)?;
            }
            CompiledTransformProperty::Rotation => {
                transform.rotation =
                    Self::compiled_property_expect_quat(value, &target.property_path)?;
            }
            CompiledTransformProperty::RotationAxis(axis) => {
                let mut rotation = transform.rotation.to_array();
                rotation[axis] =
                    Self::compiled_property_expect_scalar(value, &target.property_path)?;
                Self::compiled_property_validate_quat_array(rotation, &target.property_path)?;
                transform.rotation = Quat::from_array(rotation);
            }
            CompiledTransformProperty::Scale => {
                transform.scale =
                    Self::compiled_property_expect_vec3(value, &target.property_path)?;
            }
            CompiledTransformProperty::ScaleAxis(axis) => {
                transform.scale[axis] =
                    Self::compiled_property_expect_scalar(value, &target.property_path)?;
            }
        }

        self.update_transform(target.entity(), transform)
    }

    fn write_compiled_camera_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        property: CompiledCameraProperty,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let changed = {
            let Some(camera) = self.get_mut::<CameraComponent>(target.entity()) else {
                return Err(SceneError::MissingRequiredComponent {
                    operation: "write compiled camera property",
                    entity: target.entity(),
                    component: "Camera",
                });
            };
            let next = Self::compiled_property_expect_scalar(value, &target.property_path)?;
            let current = match property {
                CompiledCameraProperty::FovYRadians => &mut camera.fov_y_radians,
                CompiledCameraProperty::ZNear => &mut camera.z_near,
                CompiledCameraProperty::ZFar => &mut camera.z_far,
            };
            if *current == next {
                return Ok(false);
            }
            *current = next;
            true
        };
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }

    fn write_compiled_mesh_renderer_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        property: CompiledMeshRendererProperty,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let changed = {
            let Some(mesh) = self.get_mut::<MeshRenderer>(target.entity()) else {
                return Err(SceneError::MissingRequiredComponent {
                    operation: "write compiled mesh renderer property",
                    entity: target.entity(),
                    component: "MeshRenderer",
                });
            };
            match property {
                CompiledMeshRendererProperty::RenderQueue => {
                    let next = Self::compiled_property_expect_i32(value, &target.property_path)?;
                    if mesh.render_queue == next {
                        return Ok(false);
                    }
                    mesh.render_queue = next;
                }
                CompiledMeshRendererProperty::MaterialQueue => {
                    let next = Self::compiled_property_expect_i32(value, &target.property_path)?;
                    if mesh.material_queue == next {
                        return Ok(false);
                    }
                    mesh.material_queue = next;
                }
                CompiledMeshRendererProperty::OrderInLayer => {
                    let next = Self::compiled_property_expect_i32(value, &target.property_path)?;
                    if mesh.order_in_layer == next {
                        return Ok(false);
                    }
                    mesh.order_in_layer = next;
                }
                CompiledMeshRendererProperty::DepthBias => {
                    let next = Self::compiled_property_expect_scalar(value, &target.property_path)?;
                    if mesh.depth_bias == next {
                        return Ok(false);
                    }
                    mesh.depth_bias = next;
                }
                CompiledMeshRendererProperty::MorphWeight(index) => {
                    let next = Self::compiled_property_expect_scalar(value, &target.property_path)?;
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
                CompiledMeshRendererProperty::Tint => {
                    let next = Self::compiled_property_expect_vec4(value, &target.property_path)?;
                    if mesh.tint == next {
                        return Ok(false);
                    }
                    mesh.tint = next;
                }
            }
            true
        };
        if changed {
            self.mark_node_cache_dirty();
        }
        Ok(changed)
    }
}
