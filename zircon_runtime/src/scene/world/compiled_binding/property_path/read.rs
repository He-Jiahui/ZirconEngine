use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::World;
use crate::scene::components::{
    AmbientLight, CameraComponent, DirectionalLight, LocalTransform, MeshRenderer, PointLight,
    RectLight, SpotLight,
};

use super::super::compiled_scene_camera_light_fields::{
    CompiledCameraProperty, CompiledLightProperty,
};
use super::model::{
    CompiledMeshRendererProperty, CompiledScenePropertyWriter, CompiledScenePropertyWriterKind,
    CompiledTransformProperty,
};

impl World {
    /// Reads a compiled scene property without enumerating scene properties.
    ///
    /// `None` requires the caller to rebind after a hierarchy/name generation
    /// change or handle a missing Transform component explicitly.
    pub fn read_compiled_scene_property(
        &self,
        target: &CompiledScenePropertyWriter,
    ) -> Option<ScenePropertyValue> {
        if !target.is_current_for(self) {
            self.record_compiled_scene_property_stale_target();
            return None;
        }
        self.record_compiled_scene_property_reader_access();
        match &target.property {
            CompiledScenePropertyWriterKind::Transform(property) => {
                let transform = self.get::<LocalTransform>(target.entity())?.transform;
                Some(match property {
                    CompiledTransformProperty::Translation => {
                        ScenePropertyValue::Vec3(transform.translation.to_array())
                    }
                    CompiledTransformProperty::TranslationAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.translation[*axis])
                    }
                    CompiledTransformProperty::Rotation => {
                        ScenePropertyValue::Quaternion(transform.rotation.to_array())
                    }
                    CompiledTransformProperty::RotationAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.rotation.to_array()[*axis])
                    }
                    CompiledTransformProperty::Scale => {
                        ScenePropertyValue::Vec3(transform.scale.to_array())
                    }
                    CompiledTransformProperty::ScaleAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.scale[*axis])
                    }
                })
            }
            CompiledScenePropertyWriterKind::MeshRenderer(property) => {
                let mesh = self.get::<MeshRenderer>(target.entity())?;
                Some(match property {
                    CompiledMeshRendererProperty::RenderQueue => {
                        ScenePropertyValue::Integer(mesh.render_queue.into())
                    }
                    CompiledMeshRendererProperty::MaterialQueue => {
                        ScenePropertyValue::Integer(mesh.material_queue.into())
                    }
                    CompiledMeshRendererProperty::OrderInLayer => {
                        ScenePropertyValue::Integer(mesh.order_in_layer.into())
                    }
                    CompiledMeshRendererProperty::DepthBias => {
                        ScenePropertyValue::Scalar(mesh.depth_bias)
                    }
                    CompiledMeshRendererProperty::MorphWeight(index) => {
                        ScenePropertyValue::Scalar(*mesh.morph_weights.get(*index)?)
                    }
                    CompiledMeshRendererProperty::Tint => {
                        ScenePropertyValue::Vec4(mesh.tint.to_array())
                    }
                })
            }
            CompiledScenePropertyWriterKind::AnimationRuntime(property) => {
                self.read_compiled_animation_runtime_property(target.entity(), *property)
            }
            CompiledScenePropertyWriterKind::Camera(property) => {
                let camera = self.get::<CameraComponent>(target.entity())?;
                Some(match property {
                    CompiledCameraProperty::FovYRadians => {
                        ScenePropertyValue::Scalar(camera.fov_y_radians)
                    }
                    CompiledCameraProperty::ZNear => ScenePropertyValue::Scalar(camera.z_near),
                    CompiledCameraProperty::ZFar => ScenePropertyValue::Scalar(camera.z_far),
                })
            }
            CompiledScenePropertyWriterKind::Light(property) => Some(match property {
                CompiledLightProperty::AmbientColor => ScenePropertyValue::Vec3(
                    self.get::<AmbientLight>(target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::AmbientIntensity => {
                    ScenePropertyValue::Scalar(self.get::<AmbientLight>(target.entity())?.intensity)
                }
                CompiledLightProperty::AmbientAffectsLightmappedMeshes => ScenePropertyValue::Bool(
                    self.get::<AmbientLight>(target.entity())?
                        .affects_lightmapped_meshes,
                ),
                CompiledLightProperty::DirectionalDirection => ScenePropertyValue::Vec3(
                    self.get::<DirectionalLight>(target.entity())?
                        .direction
                        .to_array(),
                ),
                CompiledLightProperty::DirectionalColor => ScenePropertyValue::Vec3(
                    self.get::<DirectionalLight>(target.entity())?
                        .color
                        .to_array(),
                ),
                CompiledLightProperty::DirectionalIntensity => ScenePropertyValue::Scalar(
                    self.get::<DirectionalLight>(target.entity())?.intensity,
                ),
                CompiledLightProperty::PointColor => ScenePropertyValue::Vec3(
                    self.get::<PointLight>(target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::PointIntensity => {
                    ScenePropertyValue::Scalar(self.get::<PointLight>(target.entity())?.intensity)
                }
                CompiledLightProperty::PointRange => {
                    ScenePropertyValue::Scalar(self.get::<PointLight>(target.entity())?.range)
                }
                CompiledLightProperty::RectColor => ScenePropertyValue::Vec3(
                    self.get::<RectLight>(target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::RectIntensity => {
                    ScenePropertyValue::Scalar(self.get::<RectLight>(target.entity())?.intensity)
                }
                CompiledLightProperty::RectRange => {
                    ScenePropertyValue::Scalar(self.get::<RectLight>(target.entity())?.range)
                }
                CompiledLightProperty::RectSize => ScenePropertyValue::Vec2(
                    self.get::<RectLight>(target.entity())?.size.to_array(),
                ),
                CompiledLightProperty::SpotDirection => ScenePropertyValue::Vec3(
                    self.get::<SpotLight>(target.entity())?.direction.to_array(),
                ),
                CompiledLightProperty::SpotColor => ScenePropertyValue::Vec3(
                    self.get::<SpotLight>(target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::SpotIntensity => {
                    ScenePropertyValue::Scalar(self.get::<SpotLight>(target.entity())?.intensity)
                }
                CompiledLightProperty::SpotRange => {
                    ScenePropertyValue::Scalar(self.get::<SpotLight>(target.entity())?.range)
                }
                CompiledLightProperty::SpotInnerAngleRadians => ScenePropertyValue::Scalar(
                    self.get::<SpotLight>(target.entity())?.inner_angle_radians,
                ),
                CompiledLightProperty::SpotOuterAngleRadians => ScenePropertyValue::Scalar(
                    self.get::<SpotLight>(target.entity())?.outer_angle_radians,
                ),
            }),
            CompiledScenePropertyWriterKind::Dynamic(property) => self
                .compiled_dynamic_component_property(
                    target.entity(),
                    property.component_id(),
                    property.property(),
                ),
        }
    }
}
