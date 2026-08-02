use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::Quat;
use crate::scene::{EntityId, SceneError, SceneResult, World};

use super::compiled_scene_animation_fields::CompiledAnimationRuntimeProperty;
use super::compiled_scene_camera_light_fields::{CompiledCameraProperty, CompiledLightProperty};
use super::compiled_scene_dynamic_field::CompiledDynamicProperty;

/// Runtime-owned identity for canonical entity-path text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathId(pub(super) u64);

/// Runtime-owned identity for canonical component-property text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentFieldId(pub(super) u64);

/// A generation-validated entity/property target compiled by a [`World`].
///
/// The raw DTOs remain the serde and diagnostic owners. Runtime owns their
/// identity and the resolved entity so consumers can retain this target without
/// rebuilding path strings every frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledScenePropertyTarget {
    entity: EntityId,
    root: EntityId,
    path_id: PathId,
    component_field_id: ComponentFieldId,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompiledTransformProperty {
    Translation,
    TranslationAxis(usize),
    Rotation,
    RotationAxis(usize),
    Scale,
    ScaleAxis(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompiledMeshRendererProperty {
    RenderQueue,
    MaterialQueue,
    OrderInLayer,
    DepthBias,
    MorphWeight(usize),
    Tint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CompiledScenePropertyWriterKind {
    Transform(CompiledTransformProperty),
    MeshRenderer(CompiledMeshRendererProperty),
    AnimationRuntime(CompiledAnimationRuntimeProperty),
    Camera(CompiledCameraProperty),
    Light(CompiledLightProperty),
    Dynamic(CompiledDynamicProperty),
}

impl CompiledTransformProperty {
    fn from_canonical_key(key: &str) -> Option<Self> {
        match key {
            "transform.translation" => Some(Self::Translation),
            "transform.translation.x" | "transform.translation.0" => Some(Self::TranslationAxis(0)),
            "transform.translation.y" | "transform.translation.1" => Some(Self::TranslationAxis(1)),
            "transform.translation.z" | "transform.translation.2" => Some(Self::TranslationAxis(2)),
            "transform.rotation" => Some(Self::Rotation),
            "transform.rotation.x" | "transform.rotation.0" => Some(Self::RotationAxis(0)),
            "transform.rotation.y" | "transform.rotation.1" => Some(Self::RotationAxis(1)),
            "transform.rotation.z" | "transform.rotation.2" => Some(Self::RotationAxis(2)),
            "transform.rotation.w" | "transform.rotation.3" => Some(Self::RotationAxis(3)),
            "transform.scale" => Some(Self::Scale),
            "transform.scale.x" | "transform.scale.0" => Some(Self::ScaleAxis(0)),
            "transform.scale.y" | "transform.scale.1" => Some(Self::ScaleAxis(1)),
            "transform.scale.z" | "transform.scale.2" => Some(Self::ScaleAxis(2)),
            _ => None,
        }
    }
}

impl CompiledScenePropertyWriterKind {
    fn from_canonical_key(key: &str) -> Option<Self> {
        if let Some(property) = CompiledTransformProperty::from_canonical_key(key) {
            return Some(Self::Transform(property));
        }

        match key {
            "meshrenderer.renderqueue" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::RenderQueue,
                ));
            }
            "meshrenderer.materialqueue" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::MaterialQueue,
                ));
            }
            "meshrenderer.orderinlayer" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::OrderInLayer,
                ));
            }
            "meshrenderer.depthbias" => {
                return Some(Self::MeshRenderer(CompiledMeshRendererProperty::DepthBias));
            }
            "meshrenderer.tint" => {
                return Some(Self::MeshRenderer(CompiledMeshRendererProperty::Tint));
            }
            _ => {}
        }

        if let Some(property) = CompiledCameraProperty::from_canonical_key(key) {
            return Some(Self::Camera(property));
        }
        if let Some(property) = CompiledLightProperty::from_canonical_key(key) {
            return Some(Self::Light(property));
        }
        if let Some(property) = CompiledAnimationRuntimeProperty::from_canonical_key(key) {
            return Some(Self::AnimationRuntime(property));
        }

        let index = key
            .strip_prefix("meshrenderer.morphweights.")?
            .parse()
            .ok()?;
        Some(Self::MeshRenderer(
            CompiledMeshRendererProperty::MorphWeight(index),
        ))
    }

    fn is_current_for(&self, world: &World) -> bool {
        match self {
            Self::Dynamic(property) => property.is_current_for(world),
            Self::Transform(_)
            | Self::MeshRenderer(_)
            | Self::AnimationRuntime(_)
            | Self::Camera(_)
            | Self::Light(_) => true,
        }
    }
}

/// A typed scene-property writer compiled at an import or edit boundary.
///
/// The writer retains the raw DTO solely for typed errors. Its steady-state
/// read/write path uses a resolved entity plus a precompiled field variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledScenePropertyWriter {
    target: CompiledScenePropertyTarget,
    property_path: ComponentPropertyPath,
    property: CompiledScenePropertyWriterKind,
}

impl CompiledScenePropertyWriter {
    fn new(
        target: CompiledScenePropertyTarget,
        property_path: ComponentPropertyPath,
        property: CompiledScenePropertyWriterKind,
    ) -> Self {
        Self {
            target,
            property_path,
            property,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.target.entity()
    }

    pub(super) fn property_path(&self) -> &ComponentPropertyPath {
        &self.property_path
    }

    pub fn is_current_for(&self, world: &World) -> bool {
        self.target.is_current_for(world) && self.property.is_current_for(world)
    }
}

impl CompiledScenePropertyTarget {
    pub(super) const fn new(
        entity: EntityId,
        root: EntityId,
        path_id: PathId,
        component_field_id: ComponentFieldId,
        generation: u64,
    ) -> Self {
        Self {
            entity,
            root,
            path_id,
            component_field_id,
            generation,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn path_id(&self) -> PathId {
        self.path_id
    }

    pub const fn component_field_id(&self) -> ComponentFieldId {
        self.component_field_id
    }

    pub fn is_current_for(&self, world: &World) -> bool {
        world.contains_entity(self.entity)
            && world.contains_entity(self.root)
            && self.generation == world.scene_binding_generation(self.root)
    }
}

impl World {
    /// Compiles a scene-path target at an import or edit boundary.
    ///
    /// A retained target remains valid until the resolved entity's hierarchy or
    /// a name in its hierarchy changes. The typed field dispatch is added by the
    /// property-access cutover; this boundary intentionally does not fall back to
    /// per-frame path resolution.
    pub fn compile_scene_property_target(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> Option<CompiledScenePropertyTarget> {
        let entity = self.get_entity_by_path(entity_path)?;
        Some(self.compile_scene_property_target_for_entity(
            entity,
            entity_path.as_str(),
            property_path,
        ))
    }

    fn compile_scene_property_target_for_entity(
        &mut self,
        entity: EntityId,
        entity_identity: &str,
        property_path: &ComponentPropertyPath,
    ) -> CompiledScenePropertyTarget {
        let root = self.scene_binding_root(entity);
        let generation = self.scene_binding_generation(root);
        let path_id = self.scene_binding_generations.intern_path(entity_identity);
        let component_field_key = Self::canonical_component_field_key(property_path);
        let component_field_id = self
            .scene_binding_generations
            .intern_component_field(&component_field_key);

        CompiledScenePropertyTarget::new(entity, root, path_id, component_field_id, generation)
    }

    /// Compiles a typed scene-property writer at an import or edit boundary.
    ///
    /// Missing entities remain an optional lookup. Unsupported fields fail at
    /// the compile boundary and never fall back to the generic string visitor
    /// during steady-state application.
    pub fn compile_scene_property_writer(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let property = self.compile_scene_property_writer_kind(property_path)?;
        let Some(target) = self.compile_scene_property_target(entity_path, property_path) else {
            return Ok(None);
        };
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    /// Compiles a writer after an import boundary has already resolved its
    /// target entity and canonicalized its path. The path is interned once for
    /// stable runtime identity and never parsed during writer application.
    pub(crate) fn compile_scene_property_writer_for_entity(
        &mut self,
        entity: EntityId,
        canonical_entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let property = self.compile_scene_property_writer_kind(property_path)?;
        if !self.contains_entity(entity) {
            return Ok(None);
        }
        let target = self.compile_scene_property_target_for_entity(
            entity,
            canonical_entity_path.as_str(),
            property_path,
        );
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    fn compile_scene_property_writer_kind(
        &self,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<CompiledScenePropertyWriterKind> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        CompiledScenePropertyWriterKind::from_canonical_key(&component_field_key)
            .or_else(|| {
                CompiledDynamicProperty::compile(self, property_path)
                    .map(CompiledScenePropertyWriterKind::Dynamic)
            })
            .ok_or_else(|| SceneError::UnknownProperty {
                property_path: property_path.to_string(),
            })
    }

    /// Reads a compiled scene property without enumerating scene properties.
    ///
    /// `None` requires the caller to rebind after a hierarchy/name generation
    /// change or handle a missing Transform component explicitly.
    pub fn read_compiled_scene_property(
        &self,
        target: &CompiledScenePropertyWriter,
    ) -> Option<ScenePropertyValue> {
        if !target.is_current_for(self) {
            return None;
        }
        match &target.property {
            CompiledScenePropertyWriterKind::Transform(property) => {
                let transform = self
                    .local_transforms
                    .get(&target.entity())
                    .map(|local| local.transform)?;
                Some(match property {
                    CompiledTransformProperty::Translation => {
                        ScenePropertyValue::Vec3(transform.translation.to_array())
                    }
                    CompiledTransformProperty::TranslationAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.translation[axis])
                    }
                    CompiledTransformProperty::Rotation => {
                        ScenePropertyValue::Quaternion(transform.rotation.to_array())
                    }
                    CompiledTransformProperty::RotationAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.rotation.to_array()[axis])
                    }
                    CompiledTransformProperty::Scale => {
                        ScenePropertyValue::Vec3(transform.scale.to_array())
                    }
                    CompiledTransformProperty::ScaleAxis(axis) => {
                        ScenePropertyValue::Scalar(transform.scale[axis])
                    }
                })
            }
            CompiledScenePropertyWriterKind::MeshRenderer(property) => {
                let mesh = self.mesh_renderers.get(&target.entity())?;
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
                        ScenePropertyValue::Scalar(*mesh.morph_weights.get(index)?)
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
                let camera = self.cameras.get(&target.entity())?;
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
                    self.ambient_lights.get(&target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::AmbientIntensity => {
                    ScenePropertyValue::Scalar(self.ambient_lights.get(&target.entity())?.intensity)
                }
                CompiledLightProperty::AmbientAffectsLightmappedMeshes => ScenePropertyValue::Bool(
                    self.ambient_lights
                        .get(&target.entity())?
                        .affects_lightmapped_meshes,
                ),
                CompiledLightProperty::DirectionalDirection => ScenePropertyValue::Vec3(
                    self.directional_lights
                        .get(&target.entity())?
                        .direction
                        .to_array(),
                ),
                CompiledLightProperty::DirectionalColor => ScenePropertyValue::Vec3(
                    self.directional_lights
                        .get(&target.entity())?
                        .color
                        .to_array(),
                ),
                CompiledLightProperty::DirectionalIntensity => ScenePropertyValue::Scalar(
                    self.directional_lights.get(&target.entity())?.intensity,
                ),
                CompiledLightProperty::PointColor => ScenePropertyValue::Vec3(
                    self.point_lights.get(&target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::PointIntensity => {
                    ScenePropertyValue::Scalar(self.point_lights.get(&target.entity())?.intensity)
                }
                CompiledLightProperty::PointRange => {
                    ScenePropertyValue::Scalar(self.point_lights.get(&target.entity())?.range)
                }
                CompiledLightProperty::RectColor => ScenePropertyValue::Vec3(
                    self.rect_lights.get(&target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::RectIntensity => {
                    ScenePropertyValue::Scalar(self.rect_lights.get(&target.entity())?.intensity)
                }
                CompiledLightProperty::RectRange => {
                    ScenePropertyValue::Scalar(self.rect_lights.get(&target.entity())?.range)
                }
                CompiledLightProperty::RectSize => ScenePropertyValue::Vec2(
                    self.rect_lights.get(&target.entity())?.size.to_array(),
                ),
                CompiledLightProperty::SpotDirection => ScenePropertyValue::Vec3(
                    self.spot_lights.get(&target.entity())?.direction.to_array(),
                ),
                CompiledLightProperty::SpotColor => ScenePropertyValue::Vec3(
                    self.spot_lights.get(&target.entity())?.color.to_array(),
                ),
                CompiledLightProperty::SpotIntensity => {
                    ScenePropertyValue::Scalar(self.spot_lights.get(&target.entity())?.intensity)
                }
                CompiledLightProperty::SpotRange => {
                    ScenePropertyValue::Scalar(self.spot_lights.get(&target.entity())?.range)
                }
                CompiledLightProperty::SpotInnerAngleRadians => ScenePropertyValue::Scalar(
                    self.spot_lights.get(&target.entity())?.inner_angle_radians,
                ),
                CompiledLightProperty::SpotOuterAngleRadians => ScenePropertyValue::Scalar(
                    self.spot_lights.get(&target.entity())?.outer_angle_radians,
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

    /// Writes a compiled scene property without normalizing or dispatching a
    /// string property path during steady-state application.
    pub fn write_compiled_scene_property(
        &mut self,
        target: &CompiledScenePropertyWriter,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        if !target.is_current_for(self) {
            return Err(SceneError::PropertyUnavailable {
                entity: target.entity(),
                property_path: target.property_path.to_string(),
            });
        }
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
        let Some(local) = self.local_transforms.get(&target.entity()).copied() else {
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
            let Some(camera) = self.cameras.get_mut(&target.entity()) else {
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
            let Some(mesh) = self.mesh_renderers.get_mut(&target.entity()) else {
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

    fn scene_binding_root(&self, entity: EntityId) -> EntityId {
        let mut root = entity;
        let mut remaining = self.entities.len().saturating_add(1);
        while remaining > 0 {
            let Some(parent) = self.parent_of(root) else {
                break;
            };
            root = parent;
            remaining -= 1;
        }
        root
    }
}
