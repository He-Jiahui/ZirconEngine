//! Strict, absence-only dynamic component creation. Existing compatibility setters
//! retain their contracts; the Inspector uses this admission and batch boundary.

use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::scene::ecs::StorageType;
use crate::scene::world::typed_api::DynamicComponentPresence;
use crate::scene::world::DynamicComponentCreationId;
use crate::scene::{EntityId, SceneError, World};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicComponentCreationSpec {
    pub type_path: String,
    pub schema_generation: u64,
    /// Canonical serialization of the complete admitted registration, including defaults.
    pub schema_json: String,
    pub value: Value,
}

zircon_runtime_interface::impl_retained_owner_v1!(DynamicComponentCreationSpec {
    type_path, schema_json, value
});

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicComponentCreationTemplate {
    pub spec: DynamicComponentCreationSpec,
    pub display_name: String,
    /// Only declared defaults populate spec.value. These names require caller input.
    pub required_fields: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DynamicComponentCreationError {
    #[error(transparent)]
    Scene(#[from] SceneError),
    #[error("dynamic component type {type_path} has no editor-visible creation schema")]
    TypeUnavailable { type_path: String },
    #[error("dynamic component creation requires nonempty, distinct targets")]
    InvalidTargets,
    #[error("dynamic component schema changed for {type_path}")]
    SchemaChanged { type_path: String },
    #[error("dynamic component {type_path} already exists on {entity}")]
    AlreadyPresent { entity: EntityId, type_path: String },
    #[error("dynamic component {type_path} on {entity} is no longer the created instance and value")]
    CreationChanged { entity: EntityId, type_path: String },
    #[error("dynamic component schema serialization failed")]
    SchemaEncoding(#[from] serde_json::Error),
}

/// Compares a registration's canonical serialization with the captured bytes.
/// Successful validation neither clones the registration nor materializes an inverse.
struct SchemaBytes<'a> {
    expected: &'a [u8],
    offset: usize,
}
impl Write for SchemaBytes<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self.offset.checked_add(bytes.len())
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
        if self.expected.get(self.offset..end) != Some(bytes) {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        self.offset = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl World {
    /// Borrows the sorted dynamic catalog without copying schema/default payloads.
    /// Iteration is O(types); callers may materialize a bounded page of metadata.
    pub fn dynamic_component_creation_descriptors(&self)
        -> impl Iterator<Item = &crate::core::framework::scene::ComponentTypeDescriptor>
    {
        self.component_types.descriptors().filter(|descriptor| {
            self.type_registry.registration(&descriptor.type_id)
                .is_ok_and(|registration| registration.editor_visible)
        })
    }

    /// Returns only real declared defaults. Missing required values are never synthesized.
    pub fn dynamic_component_creation_template(
        &self, type_path: &str,
    ) -> Result<DynamicComponentCreationTemplate, DynamicComponentCreationError> {
        let descriptor = self.component_type_descriptor(type_path).ok_or_else(|| {
            DynamicComponentCreationError::TypeUnavailable { type_path: type_path.to_owned() }
        })?;
        let registration = self.type_registry.registration(type_path)
            .map_err(SceneError::from)?;
        if !registration.editor_visible {
            return Err(DynamicComponentCreationError::TypeUnavailable { type_path: type_path.to_owned() });
        }
        let mut value = Map::new();
        let mut required_fields = Vec::new();
        for field in &registration.type_info.fields {
            if let Some(default) = &field.default_value {
                let json = crate::scene::reflect::json_value_from_reflected(default.clone())
                    .map_err(SceneError::from)?;
                value.insert(field.name.clone(), json);
            } else {
                required_fields.push(field.name.clone());
            }
        }
        Ok(DynamicComponentCreationTemplate {
            spec: DynamicComponentCreationSpec {
                type_path: type_path.to_owned(),
                schema_generation: self.component_type_schema_generation(type_path),
                schema_json: serde_json::to_string(registration)?,
                value: Value::Object(value),
            },
            display_name: descriptor.display_name.clone(),
            required_fields,
        })
    }

    pub fn validate_dynamic_component_creation(
        &self, targets: &[EntityId], spec: &DynamicComponentCreationSpec,
    ) -> Result<(), DynamicComponentCreationError> {
        self.validate_dynamic_component_creation_inner(targets, spec, true)
    }

    /// Explicit journal replay may rebind the ephemeral catalog generation only
    /// after the entire persisted registration still matches byte for byte.
    pub fn validate_dynamic_component_creation_replay(
        &self, targets: &[EntityId], spec: &DynamicComponentCreationSpec,
    ) -> Result<(), DynamicComponentCreationError> {
        self.validate_dynamic_component_creation_inner(targets, spec, false)
    }

    fn validate_dynamic_component_creation_inner(
        &self, targets: &[EntityId], spec: &DynamicComponentCreationSpec, check_generation: bool,
    ) -> Result<(), DynamicComponentCreationError> {
        self.validate_creation_targets(targets)?;
        if self.component_type_descriptor(&spec.type_path).is_none() {
            return Err(DynamicComponentCreationError::TypeUnavailable { type_path: spec.type_path.clone() });
        }
        let registration = self.type_registry.registration(&spec.type_path)
            .map_err(SceneError::from)?;
        let mut bytes = SchemaBytes { expected: spec.schema_json.as_bytes(), offset: 0 };
        if !registration.editor_visible
            || (check_generation && self.component_type_schema_generation(&spec.type_path) != spec.schema_generation)
            || serde_json::to_writer(&mut bytes, registration).is_err()
            || bytes.offset != bytes.expected.len()
        {
            return Err(DynamicComponentCreationError::SchemaChanged { type_path: spec.type_path.clone() });
        }
        Self::validate_dynamic_component_value_against_registration(registration, &spec.value)?;
        for &entity in targets {
            if self.dynamic_component(entity, &spec.type_path).is_some() {
                return Err(DynamicComponentCreationError::AlreadyPresent { entity, type_path: spec.type_path.clone() });
            }
            if let Some(id) = self.registered_dynamic_component_id(&spec.type_path) {
                if self.contains_component_id(entity, id) {
                    return Err(DynamicComponentCreationError::AlreadyPresent { entity, type_path: spec.type_path.clone() });
                }
                self.component_storage.preflight_insert::<DynamicComponentPresence>(id, StorageType::SparseSet)
                    .map_err(SceneError::from)?;
            }
        }
        Ok(())
    }

    fn validate_creation_targets(&self, targets: &[EntityId]) -> Result<(), DynamicComponentCreationError> {
        if targets.is_empty() || targets.iter().enumerate().any(|(index, target)| targets[..index].contains(target)) {
            return Err(DynamicComponentCreationError::InvalidTargets);
        }
        for &entity in targets {
            if !self.contains_entity(entity) {
                return Err(SceneError::missing_entity("create dynamic component on", entity).into());
            }
        }
        Ok(())
    }

    /// One exclusive World borrow, full batch preflight, then no fallible admission or callbacks.
    /// As with the existing JSON setter, presence publication does not invoke native factories.
    pub fn create_dynamic_components(
        &mut self, targets: &[EntityId], spec: &DynamicComponentCreationSpec,
    ) -> Result<Vec<DynamicComponentCreationId>, DynamicComponentCreationError> {
        self.validate_dynamic_component_creation(targets, spec)?;
        let component_id = self.ensure_dynamic_component_presence_id(&spec.type_path)?;
        self.component_storage.preflight_insert::<DynamicComponentPresence>(component_id, StorageType::SparseSet)
            .map_err(SceneError::from)?;
        // Each real instance receives a fresh allocation. Do not clone one Default
        // marker for unrelated targets. Stage all owned values before publication.
        let mut pending = Vec::with_capacity(targets.len());
        let mut receipts = Vec::with_capacity(targets.len());
        for &entity in targets {
            let presence = DynamicComponentPresence::default();
            receipts.push(presence.creation.clone());
            let mut row = self.begin_component_row(entity);
            self.stage_component_row_value_with_id(&mut row, component_id, presence);
            pending.push((entity, spec.value.clone(), row));
        }
        for (entity, value, row) in pending {
            self.dynamic_components.entry(entity).or_default().insert(spec.type_path.clone(), value);
            self.commit_component_row(entity, row, false);
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        self.bump_lifecycle_visibility_revision();
        self.advance_dynamic_component_generation(&spec.type_path);
        self.advance_world_generation();
        self.invalidate_world_component_type(&spec.type_path);
        Ok(receipts)
    }

    pub fn validate_created_dynamic_components(
        &self, targets: &[EntityId], spec: &DynamicComponentCreationSpec,
        receipts: &[DynamicComponentCreationId],
    ) -> Result<(), DynamicComponentCreationError> {
        self.validate_creation_targets(targets)?;
        if targets.len() != receipts.len() {
            return Err(DynamicComponentCreationError::InvalidTargets);
        }
        let id = self.registered_dynamic_component_id(&spec.type_path)
            .ok_or_else(|| DynamicComponentCreationError::TypeUnavailable { type_path: spec.type_path.clone() })?;
        self.component_storage.preflight_insert::<DynamicComponentPresence>(id, StorageType::SparseSet)
            .map_err(SceneError::from)?;
        for (&entity, receipt) in targets.iter().zip(receipts) {
            let presence = self.internal_entity(entity)
                .and_then(|internal| self.component_storage.get::<DynamicComponentPresence>(id, internal));
            if self.dynamic_component(entity, &spec.type_path) != Some(&spec.value)
                || !presence.is_some_and(|presence| presence.creation == *receipt)
            {
                return Err(DynamicComponentCreationError::CreationChanged { entity, type_path: spec.type_path.clone() });
            }
        }
        Ok(())
    }

    /// Removes only this command's unchanged instances. A same-JSON remove/readd,
    /// projection rebuild, or other creation receipt is rejected for the whole batch.
    pub fn remove_created_dynamic_components(
        &mut self, targets: &[EntityId], spec: &DynamicComponentCreationSpec,
        receipts: &[DynamicComponentCreationId],
    ) -> Result<(), DynamicComponentCreationError> {
        self.validate_created_dynamic_components(targets, spec, receipts)?;
        // Typed storage and all rows were proven above; no callback can intervene.
        for &entity in targets {
            self.remove_dynamic_component_presence(entity, &spec.type_path)
                .expect("preflighted dynamic presence removal must retain its storage type");
            let components = self.dynamic_components.get_mut(&entity)
                .expect("preflighted dynamic component map must remain present");
            components.remove(&spec.type_path);
            if components.is_empty() { self.dynamic_components.remove(&entity); }
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        self.advance_dynamic_component_generation(&spec.type_path);
        self.advance_world_generation();
        self.invalidate_world_component_type(&spec.type_path);
        Ok(())
    }
}
