//! VM 状态快照以稳定类型和字段身份跨代际传递；结构校验约束重复字段与值预算，迁移器只在此协议内重映射。
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{ReflectFieldId, ReflectTypePath, ReflectedValue};

use super::state_migration::VmStateMigrationError;
use crate::scene::reflect::RUNTIME_REFLECT_VALUE_BUDGET;

/// Current schema version emitted by stable-field-ID VM state snapshots.
pub const VM_STATE_SCHEMA_VERSION_V3: u32 = 3;

/// Stable identity for one reflected type present in a VM state snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmStateTypeIdentity {
    /// Fully qualified type path shared with the runtime reflection registry.
    pub type_path: ReflectTypePath,
    /// Producer-defined structural hash used to identify the type revision.
    pub type_hash: u32,
}

/// One stable-ID-addressed value in a reflected VM state object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VmStateFieldValue {
    /// Stable field identity shared with the runtime reflection schema.
    pub field_id: ReflectFieldId,
    /// Serialized field value.
    pub value: ReflectedValue,
}

impl VmStateFieldValue {
    pub fn new(field_id: ReflectFieldId, value: ReflectedValue) -> Self {
        Self { field_id, value }
    }
}

/// One reflected state object encoded inside a `VmStateBlob` payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VmStateObject {
    /// Reflected type path declared by the blob's authoritative type table.
    pub type_path: ReflectTypePath,
    /// Ordered reflected field values for this object.
    pub fields: Vec<VmStateFieldValue>,
}

/// Versioned VM state snapshot supporting either opaque or reflected payloads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VmStateBlob {
    /// Snapshot schema version.
    pub schema_version: u32,
    /// Authoritative type identities for reflected payload objects.
    pub types: Vec<VmStateTypeIdentity>,
    /// Opaque backend bytes or an encoded `Vec<VmStateObject>`.
    pub payload: Vec<u8>,
}

impl VmStateBlob {
    /// Creates an opaque default-version snapshot that does not opt into reflected migration.
    pub fn from_payload(payload: Vec<u8>) -> Self {
        Self {
            payload,
            ..Self::default()
        }
    }

    /// Decodes a complete versioned snapshot from the VM lifecycle JSON protocol.
    pub fn from_json(snapshot: &str) -> Result<Self, VmStateMigrationError> {
        let blob = serde_json::from_str::<Self>(snapshot).map_err(|error| {
            VmStateMigrationError::SnapshotDecode {
                reason: error.to_string(),
            }
        })?;
        if !blob.types.is_empty() {
            blob.validate_reflected()?;
        }
        Ok(blob)
    }

    /// Encodes a complete versioned snapshot for the VM lifecycle JSON protocol.
    pub fn to_json(&self) -> Result<String, VmStateMigrationError> {
        if !self.types.is_empty() {
            self.validate_reflected()?;
        }
        serde_json::to_string(self).map_err(|error| VmStateMigrationError::SnapshotEncode {
            reason: error.to_string(),
        })
    }

    /// Encodes reflected objects after validating the authoritative type table.
    pub fn from_reflected_objects(
        schema_version: u32,
        types: Vec<VmStateTypeIdentity>,
        objects: &[VmStateObject],
    ) -> Result<Self, VmStateMigrationError> {
        validate_reflected_objects(&types, objects)?;
        let payload =
            serde_json::to_vec(objects).map_err(|error| VmStateMigrationError::PayloadEncode {
                reason: error.to_string(),
            })?;
        Ok(Self {
            schema_version,
            types,
            payload,
        })
    }

    /// Decodes and validates reflected objects against the authoritative type table.
    pub fn reflected_objects(&self) -> Result<Vec<VmStateObject>, VmStateMigrationError> {
        let objects = if self.payload.is_empty() {
            Vec::new()
        } else {
            serde_json::from_slice(&self.payload).map_err(|error| {
                VmStateMigrationError::PayloadDecode {
                    reason: error.to_string(),
                }
            })?
        };
        validate_reflected_objects(&self.types, &objects)?;
        Ok(objects)
    }

    /// Validates that this blob is a well-formed reflected snapshot.
    pub fn validate_reflected(&self) -> Result<(), VmStateMigrationError> {
        self.reflected_objects().map(|_| ())
    }
}

fn validate_reflected_objects(
    types: &[VmStateTypeIdentity],
    objects: &[VmStateObject],
) -> Result<(), VmStateMigrationError> {
    let mut type_paths = HashSet::with_capacity(types.len());
    for identity in types {
        let type_path = identity.type_path.type_path();
        if !type_paths.insert(type_path) {
            return Err(VmStateMigrationError::DuplicateSourceTypeIdentity {
                type_path: type_path.to_string(),
            });
        }
    }
    for object in objects {
        let type_path = object.type_path.type_path();
        if !type_paths.contains(type_path) {
            return Err(VmStateMigrationError::MissingSourceTypeIdentity {
                type_path: type_path.to_string(),
            });
        }
        let mut field_ids = HashSet::with_capacity(object.fields.len());
        for field in &object.fields {
            if !field_ids.insert(field.field_id) {
                return Err(VmStateMigrationError::DuplicateSourceField {
                    type_path: type_path.to_string(),
                    field: field.field_id.to_string(),
                });
            }
            field
                .value
                .validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)
                .map_err(|error| VmStateMigrationError::ReflectedValueRejected {
                    type_path: type_path.to_string(),
                    field: field.field_id.to_string(),
                    error,
                })?;
        }
    }
    Ok(())
}

impl Default for VmStateBlob {
    fn default() -> Self {
        Self {
            schema_version: VM_STATE_SCHEMA_VERSION_V3,
            types: Vec::new(),
            payload: Vec::new(),
        }
    }
}

#[cfg(test)]
#[path = "tests/vm_state_blob.rs"]
mod tests;
