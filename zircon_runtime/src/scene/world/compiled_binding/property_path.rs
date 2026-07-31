use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::Quat;
use crate::scene::{EntityId, SceneError, SceneResult, World};

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

/// A typed Transform read target that never revisits generic property dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledTransformPropertyTarget {
    target: CompiledScenePropertyTarget,
    property_path: ComponentPropertyPath,
    property: CompiledTransformProperty,
}

impl CompiledTransformPropertyTarget {
    fn new(
        target: CompiledScenePropertyTarget,
        property_path: ComponentPropertyPath,
        property: CompiledTransformProperty,
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

    pub fn is_current_for(&self, world: &World) -> bool {
        self.target.is_current_for(world)
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
        let root = self.scene_binding_root(entity);
        let generation = self.scene_binding_generation(root);
        let path_id = self
            .scene_binding_generations
            .intern_path(entity_path.as_str());
        let component_field_key = Self::canonical_component_field_key(property_path);
        let component_field_id = self
            .scene_binding_generations
            .intern_component_field(&component_field_key);

        Some(CompiledScenePropertyTarget::new(
            entity,
            root,
            path_id,
            component_field_id,
            generation,
        ))
    }

    /// Compiles a direct Transform read target at an import or edit boundary.
    ///
    /// It intentionally returns `None` for unsupported fields rather than
    /// redirecting to the generic string property visitor.
    pub fn compile_transform_property_target(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> Option<CompiledTransformPropertyTarget> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        let property = CompiledTransformProperty::from_canonical_key(&component_field_key)?;
        let target = self.compile_scene_property_target(entity_path, property_path)?;
        Some(CompiledTransformPropertyTarget::new(
            target,
            property_path.clone(),
            property,
        ))
    }

    /// Reads a compiled Transform target without enumerating scene properties.
    ///
    /// `None` requires the caller to rebind after a hierarchy/name generation
    /// change or handle a missing Transform component explicitly.
    pub fn read_compiled_transform_property(
        &self,
        target: &CompiledTransformPropertyTarget,
    ) -> Option<ScenePropertyValue> {
        if !target.is_current_for(self) {
            return None;
        }
        let transform = self
            .local_transforms
            .get(&target.entity())
            .map(|local| local.transform)?;

        Some(match target.property {
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

    /// Writes a compiled Transform target without normalizing or dispatching a
    /// string property path during steady-state application.
    pub fn write_compiled_transform_property(
        &mut self,
        target: &CompiledTransformPropertyTarget,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        if !target.is_current_for(self) {
            return Err(SceneError::PropertyUnavailable {
                entity: target.entity(),
                property_path: target.property_path.to_string(),
            });
        }
        let Some(local) = self.local_transforms.get(&target.entity()).copied() else {
            return Err(SceneError::MissingRequiredComponent {
                operation: "write compiled transform property",
                entity: target.entity(),
                component: "LocalTransform",
            });
        };

        let mut transform = local.transform;
        match target.property {
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
