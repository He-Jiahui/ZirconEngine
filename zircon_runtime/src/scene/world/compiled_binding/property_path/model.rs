use crate::core::framework::scene::ComponentPropertyPath;
use crate::scene::{EntityId, World};

use super::super::compiled_scene_animation_fields::CompiledAnimationRuntimeProperty;
use super::super::compiled_scene_camera_light_fields::{
    CompiledCameraProperty, CompiledLightProperty,
};
use super::super::compiled_scene_dynamic_field::CompiledDynamicProperty;

/// Runtime-owned identity for canonical entity-path text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathId(pub(in crate::scene::world::compiled_binding) u64);

/// Runtime-owned identity for canonical component-property text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentFieldId(pub(in crate::scene::world::compiled_binding) u64);

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
pub(super) enum CompiledTransformProperty {
    Translation,
    TranslationAxis(usize),
    Rotation,
    RotationAxis(usize),
    Scale,
    ScaleAxis(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledMeshRendererProperty {
    RenderQueue,
    MaterialQueue,
    OrderInLayer,
    DepthBias,
    MorphWeight(usize),
    Tint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CompiledScenePropertyWriterKind {
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
    pub(super) fn from_canonical_key(key: &str) -> Option<Self> {
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
    pub(super) property_path: ComponentPropertyPath,
    pub(super) property: CompiledScenePropertyWriterKind,
}

impl CompiledScenePropertyWriter {
    pub(super) fn new(
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

    pub(in crate::scene::world::compiled_binding) fn property_path(
        &self,
    ) -> &ComponentPropertyPath {
        &self.property_path
    }

    pub fn is_current_for(&self, world: &World) -> bool {
        self.target.is_current_for(world) && self.property.is_current_for(world)
    }
}

impl CompiledScenePropertyTarget {
    pub(in crate::scene::world::compiled_binding) const fn new(
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
