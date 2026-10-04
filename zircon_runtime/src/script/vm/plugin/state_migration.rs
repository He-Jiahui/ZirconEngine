//! 状态迁移按稳定类型和字段 ID 把旧快照投影到新 schema，并应用新字段默认值；不会替后端解释任意类型转换。
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::reflect::{
    ReflectFieldId, ReflectTypeRegistration, ReflectValueValidationError,
};

use crate::scene::reflect::RUNTIME_REFLECT_VALUE_BUDGET;

use super::{VmStateBlob, VmStateFieldValue, VmStateObject, VmStateTypeIdentity};

/// Target reflected type registration plus its structural revision identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VmStateTypeSchema {
    /// Shared reflection registration consumed by every engine subsystem.
    pub registration: ReflectTypeRegistration,
    /// Structural hash written to the migrated type identity table.
    pub type_hash: u32,
}

/// Destination schema published by a VM plugin generation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VmStateSchema {
    /// Schema version written to the migrated snapshot.
    pub schema_version: u32,
    /// Serializable target type registrations.
    pub types: Vec<VmStateTypeSchema>,
}

impl VmStateSchema {
    /// Decodes a schema published by a VM lifecycle `stateSchema` export.
    pub fn from_json(schema: &str) -> Result<Self, VmStateMigrationError> {
        let schema: Self =
            serde_json::from_str(schema).map_err(|error| VmStateMigrationError::SchemaDecode {
                reason: error.to_string(),
            })?;
        validate_schema_default_values(&schema)?;
        Ok(schema)
    }

    /// Encodes a schema for a VM lifecycle `stateSchema` export.
    pub fn to_json(&self) -> Result<String, VmStateMigrationError> {
        validate_schema_default_values(self)?;
        serde_json::to_string(self).map_err(|error| VmStateMigrationError::SchemaEncode {
            reason: error.to_string(),
        })
    }
}

/// Typed validation, encoding, and field-migration failures.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum VmStateMigrationError {
    /// Reflected object payload could not be decoded.
    #[error("vm state payload decode failed: {reason}")]
    PayloadDecode { reason: String },
    /// Reflected object payload could not be encoded.
    #[error("vm state payload encode failed: {reason}")]
    PayloadEncode { reason: String },
    /// A VM-provided target schema could not be decoded.
    #[error("vm state schema decode failed: {reason}")]
    SchemaDecode { reason: String },
    /// A VM-provided target schema could not be encoded.
    #[error("vm state schema encode failed: {reason}")]
    SchemaEncode { reason: String },
    /// A complete versioned snapshot could not be decoded.
    #[error("vm state snapshot decode failed: {reason}")]
    SnapshotDecode { reason: String },
    /// A complete versioned snapshot could not be encoded.
    #[error("vm state snapshot encode failed: {reason}")]
    SnapshotEncode { reason: String },
    /// Two target registrations use the same fully qualified type path.
    #[error("duplicate target vm state type `{type_path}`")]
    DuplicateTargetType { type_path: String },
    /// Two source identities use the same fully qualified type path.
    #[error("duplicate source vm state type identity `{type_path}`")]
    DuplicateSourceTypeIdentity { type_path: String },
    /// A reflected payload object is absent from the source identity table.
    #[error("vm state payload contains undeclared source type `{type_path}`")]
    MissingSourceTypeIdentity { type_path: String },
    /// The destination schema cannot accept a source object type.
    #[error("target vm state schema does not contain source type `{type_path}`")]
    MissingTargetType { type_path: String },
    /// A destination type opted out of serialization.
    #[error("target vm state type `{type_path}` is not serializable")]
    NonSerializableTargetType { type_path: String },
    /// A source object contains a duplicate stable field identity.
    #[error("vm state object `{type_path}` contains duplicate field `{field}`")]
    DuplicateSourceField { type_path: String, field: String },
    /// A reflected source value or target default exceeds runtime value admission.
    #[error("vm state value `{field}` on `{type_path}` was rejected: {error}")]
    ReflectedValueRejected {
        type_path: String,
        field: String,
        error: ReflectValueValidationError,
    },
    /// A target reflection registration contains a duplicate serializable field.
    #[error("vm state type `{type_path}` contains duplicate target field `{field}`")]
    DuplicateTargetField { type_path: String, field: String },
    /// A required target field has no current value, historical value, or default.
    #[error("vm state type `{type_path}` is missing required field `{field}`")]
    MissingRequiredField { type_path: String, field: String },
}

/// Migrates a reflected snapshot into the destination schema without value coercion.
pub fn migrate_vm_state_blob(
    source: &VmStateBlob,
    target: &VmStateSchema,
) -> Result<VmStateBlob, VmStateMigrationError> {
    let target_types = index_target_types(target)?;
    let mut migrated_objects = Vec::new();
    for object in source.reflected_objects()? {
        let type_path = object.type_path.type_path().to_string();
        let target_type = target_types
            .get(type_path.as_str())
            .copied()
            .ok_or_else(|| VmStateMigrationError::MissingTargetType {
                type_path: type_path.clone(),
            })?;
        migrated_objects.push(migrate_object(object, target_type)?);
    }

    let identities = target
        .types
        .iter()
        .map(|schema| VmStateTypeIdentity {
            type_path: schema.registration.type_path.clone(),
            type_hash: schema.type_hash,
        })
        .collect();
    VmStateBlob::from_reflected_objects(target.schema_version, identities, &migrated_objects)
}

fn index_target_types(
    target: &VmStateSchema,
) -> Result<HashMap<&str, &VmStateTypeSchema>, VmStateMigrationError> {
    let mut target_types = HashMap::with_capacity(target.types.len());
    for target_type in &target.types {
        let type_path = target_type.registration.type_path.type_path();
        if !target_type.registration.serializable {
            return Err(VmStateMigrationError::NonSerializableTargetType {
                type_path: type_path.to_string(),
            });
        }
        if target_types.insert(type_path, target_type).is_some() {
            return Err(VmStateMigrationError::DuplicateTargetType {
                type_path: type_path.to_string(),
            });
        }
        validate_target_default_values(target_type)?;
    }
    Ok(target_types)
}

fn migrate_object(
    object: VmStateObject,
    target: &VmStateTypeSchema,
) -> Result<VmStateObject, VmStateMigrationError> {
    let mut source_fields = HashMap::with_capacity(object.fields.len());
    for field in object.fields {
        source_fields.insert(field.field_id, field.value);
    }

    validate_target_fields(target)?;
    let target_fields = target
        .registration
        .type_info
        .fields
        .iter()
        .filter(|field| field.serializable)
        .collect::<Vec<_>>();
    let mut fields = Vec::with_capacity(target_fields.len());
    for field in target_fields {
        let value = source_fields.remove(&field.id);
        let value = match value {
            Some(value) => value,
            None => field.default_value.clone().ok_or_else(|| {
                VmStateMigrationError::MissingRequiredField {
                    type_path: target.registration.type_path.type_path().to_string(),
                    field: field.name.clone(),
                }
            })?,
        };
        fields.push(VmStateFieldValue::new(field.id, value));
    }

    Ok(VmStateObject {
        type_path: target.registration.type_path.clone(),
        fields,
    })
}

fn validate_target_fields(
    target: &VmStateTypeSchema,
) -> Result<HashSet<ReflectFieldId>, VmStateMigrationError> {
    let mut ids = HashSet::with_capacity(target.registration.type_info.fields.len());
    for field in target
        .registration
        .type_info
        .fields
        .iter()
        .filter(|field| field.serializable)
    {
        if !ids.insert(field.id) {
            return Err(VmStateMigrationError::DuplicateTargetField {
                type_path: target.registration.type_path.type_path().to_string(),
                field: field.id.to_string(),
            });
        }
    }
    Ok(ids)
}

fn validate_schema_default_values(schema: &VmStateSchema) -> Result<(), VmStateMigrationError> {
    for target in &schema.types {
        validate_target_default_values(target)?;
    }
    Ok(())
}

fn validate_target_default_values(target: &VmStateTypeSchema) -> Result<(), VmStateMigrationError> {
    for field in &target.registration.type_info.fields {
        let Some(default_value) = &field.default_value else {
            continue;
        };
        default_value
            .validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)
            .map_err(|error| VmStateMigrationError::ReflectedValueRejected {
                type_path: target.registration.type_path.type_path().to_string(),
                field: field.name.clone(),
                error,
            })?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/state_migration.rs"]
mod tests;
