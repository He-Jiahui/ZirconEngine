use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectTypePath};

use super::state_migration::VmStateMigrationError;

/// Current schema version emitted by opaque/default VM state snapshots.
pub const VM_STATE_SCHEMA_VERSION_V2: u32 = 2;

/// Stable identity for one reflected type present in a VM state snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmStateTypeIdentity {
    /// Fully qualified type path shared with the runtime reflection registry.
    pub type_path: ReflectTypePath,
    /// Producer-defined structural hash used to identify the type revision.
    pub type_hash: u32,
}

/// One reflected state object encoded inside a `VmStateBlob` payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VmStateObject {
    /// Reflected type path declared by the blob's authoritative type table.
    pub type_path: ReflectTypePath,
    /// Ordered reflected field values for this object.
    pub fields: Vec<ReflectFieldValue>,
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
    /// Creates an opaque v2 snapshot that does not opt into reflected migration.
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
    let mut type_paths = BTreeSet::new();
    for identity in types {
        let type_path = identity.type_path.type_path.as_str();
        if !type_paths.insert(type_path) {
            return Err(VmStateMigrationError::DuplicateSourceTypeIdentity {
                type_path: type_path.to_string(),
            });
        }
    }
    for object in objects {
        let type_path = object.type_path.type_path.as_str();
        if !type_paths.contains(type_path) {
            return Err(VmStateMigrationError::MissingSourceTypeIdentity {
                type_path: type_path.to_string(),
            });
        }
        let mut field_names = BTreeSet::new();
        for field in &object.fields {
            if !field_names.insert(field.field_name.as_str()) {
                return Err(VmStateMigrationError::DuplicateSourceField {
                    type_path: type_path.to_string(),
                    field: field.field_name.clone(),
                });
            }
        }
    }
    Ok(())
}

impl Default for VmStateBlob {
    fn default() -> Self {
        Self {
            schema_version: VM_STATE_SCHEMA_VERSION_V2,
            types: Vec::new(),
            payload: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectTypePath, ReflectedValue};

    use super::{VmStateBlob, VmStateObject, VmStateTypeIdentity};

    #[test]
    fn state_blob_round_trips_with_schema() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let state = VmStateBlob::from_reflected_objects(
            2,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 0xA11C_E001,
            }],
            &[VmStateObject {
                type_path,
                fields: vec![ReflectFieldValue::new(
                    "health",
                    ReflectedValue::Scalar(75.0),
                )],
            }],
        )
        .unwrap();

        let encoded = state.to_json().unwrap();
        let decoded = VmStateBlob::from_json(&encoded).unwrap();

        assert_eq!(decoded, state);
    }
}
