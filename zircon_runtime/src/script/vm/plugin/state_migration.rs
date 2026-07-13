use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectTypeRegistration};

use super::{VmStateBlob, VmStateObject, VmStateTypeIdentity};

/// Historical field-name mapping applied while migrating one reflected type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmStateFieldRename {
    /// Field name stored by the source schema.
    pub from: String,
    /// Serializable field name in the target registration.
    pub to: String,
}

/// Target reflected type registration plus migration-only revision metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VmStateTypeSchema {
    /// Shared reflection registration consumed by every engine subsystem.
    pub registration: ReflectTypeRegistration,
    /// Structural hash written to the migrated type identity table.
    pub type_hash: u32,
    /// Historical field mappings for this type revision.
    pub renames: Vec<VmStateFieldRename>,
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
        serde_json::from_str(schema).map_err(|error| VmStateMigrationError::SchemaDecode {
            reason: error.to_string(),
        })
    }

    /// Encodes a schema for a VM lifecycle `stateSchema` export.
    pub fn to_json(&self) -> Result<String, VmStateMigrationError> {
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
    /// A source object contains a duplicate field name.
    #[error("vm state object `{type_path}` contains duplicate field `{field}`")]
    DuplicateSourceField { type_path: String, field: String },
    /// A target reflection registration contains a duplicate serializable field.
    #[error("vm state type `{type_path}` contains duplicate target field `{field}`")]
    DuplicateTargetField { type_path: String, field: String },
    /// Two rename declarations consume the same source name.
    #[error("vm state type `{type_path}` contains duplicate rename source `{field}`")]
    DuplicateRenameSource { type_path: String, field: String },
    /// Two rename declarations write the same target name.
    #[error("vm state type `{type_path}` contains duplicate rename target `{field}`")]
    DuplicateRenameTarget { type_path: String, field: String },
    /// A rename declaration targets a non-serializable or absent field.
    #[error("vm state type `{type_path}` rename target `{field}` is not in the target schema")]
    UnknownRenameTarget { type_path: String, field: String },
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
        let type_path = object.type_path.type_path.clone();
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
) -> Result<BTreeMap<&str, &VmStateTypeSchema>, VmStateMigrationError> {
    let mut target_types = BTreeMap::new();
    for target_type in &target.types {
        let type_path = target_type.registration.type_path.type_path.as_str();
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
    }
    Ok(target_types)
}

fn migrate_object(
    object: VmStateObject,
    target: &VmStateTypeSchema,
) -> Result<VmStateObject, VmStateMigrationError> {
    let mut source_fields = BTreeMap::new();
    for field in object.fields {
        source_fields.insert(field.field_name, field.value);
    }

    let target_field_names = validate_target_fields(target)?;
    let renames = validate_renames(target, &target_field_names)?;
    let target_fields = target
        .registration
        .type_info
        .fields
        .iter()
        .filter(|field| field.serializable)
        .collect::<Vec<_>>();
    let mut fields = Vec::with_capacity(target_fields.len());
    for field in target_fields {
        let value = source_fields.remove(&field.name).or_else(|| {
            renames
                .get(field.name.as_str())
                .and_then(|old_name| source_fields.remove(*old_name))
        });
        let value = match value {
            Some(value) => value,
            None => field.default_value.clone().ok_or_else(|| {
                VmStateMigrationError::MissingRequiredField {
                    type_path: target.registration.type_path.type_path.clone(),
                    field: field.name.clone(),
                }
            })?,
        };
        fields.push(ReflectFieldValue::new(field.name.clone(), value));
    }

    Ok(VmStateObject {
        type_path: target.registration.type_path.clone(),
        fields,
    })
}

fn validate_target_fields(
    target: &VmStateTypeSchema,
) -> Result<BTreeSet<&str>, VmStateMigrationError> {
    let mut names = BTreeSet::new();
    for field in target
        .registration
        .type_info
        .fields
        .iter()
        .filter(|field| field.serializable)
    {
        if !names.insert(field.name.as_str()) {
            return Err(VmStateMigrationError::DuplicateTargetField {
                type_path: target.registration.type_path.type_path.clone(),
                field: field.name.clone(),
            });
        }
    }
    Ok(names)
}

fn validate_renames<'a>(
    target: &'a VmStateTypeSchema,
    target_fields: &BTreeSet<&str>,
) -> Result<BTreeMap<&'a str, &'a str>, VmStateMigrationError> {
    let mut sources = BTreeSet::new();
    let mut targets = BTreeMap::new();
    for rename in &target.renames {
        if !sources.insert(rename.from.as_str()) {
            return Err(VmStateMigrationError::DuplicateRenameSource {
                type_path: target.registration.type_path.type_path.clone(),
                field: rename.from.clone(),
            });
        }
        if !target_fields.contains(rename.to.as_str()) {
            return Err(VmStateMigrationError::UnknownRenameTarget {
                type_path: target.registration.type_path.type_path.clone(),
                field: rename.to.clone(),
            });
        }
        if targets
            .insert(rename.to.as_str(), rename.from.as_str())
            .is_some()
        {
            return Err(VmStateMigrationError::DuplicateRenameTarget {
                type_path: target.registration.type_path.type_path.clone(),
                field: rename.to.clone(),
            });
        }
    }
    Ok(targets)
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectFieldInfo, ReflectFieldValue, ReflectSerializationStrategy,
        ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
    };

    use super::{
        migrate_vm_state_blob, VmStateFieldRename, VmStateMigrationError, VmStateSchema,
        VmStateTypeSchema,
    };
    use crate::script::{VmStateBlob, VmStateObject, VmStateTypeIdentity};

    #[test]
    fn schema_change_migrates_fields() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let source = VmStateBlob::from_reflected_objects(
            1,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 1,
            }],
            &[VmStateObject {
                type_path: type_path.clone(),
                fields: vec![
                    ReflectFieldValue::new("old_health", ReflectedValue::Scalar(75.0)),
                    ReflectFieldValue::new("name", ReflectedValue::String("Ada".to_string())),
                    ReflectFieldValue::new("removed", ReflectedValue::Bool(true)),
                ],
            }],
        )
        .unwrap();
        let target = VmStateSchema {
            schema_version: 2,
            types: vec![VmStateTypeSchema {
                registration: ReflectTypeRegistration::new(
                    type_path.clone(),
                    "Player State",
                    ReflectTypeInfo::struct_with_fields(vec![
                        ReflectFieldInfo::new("health", "f64", ReflectEditorHint::Scalar),
                        ReflectFieldInfo::new("max_health", "f64", ReflectEditorHint::Scalar)
                            .with_default_value(ReflectedValue::Scalar(100.0)),
                        ReflectFieldInfo::new("name", "String", ReflectEditorHint::String),
                    ]),
                    ReflectSerializationStrategy::Value,
                ),
                type_hash: 2,
                renames: vec![VmStateFieldRename {
                    from: "old_health".to_string(),
                    to: "health".to_string(),
                }],
            }],
        };

        let migrated = migrate_vm_state_blob(&source, &target).unwrap();

        assert_eq!(migrated.schema_version, 2);
        assert_eq!(
            migrated.types,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 2,
            }]
        );
        assert_eq!(
            migrated.reflected_objects().unwrap(),
            vec![VmStateObject {
                type_path,
                fields: vec![
                    ReflectFieldValue::new("health", ReflectedValue::Scalar(75.0)),
                    ReflectFieldValue::new("max_health", ReflectedValue::Scalar(100.0)),
                    ReflectFieldValue::new("name", ReflectedValue::String("Ada".to_string())),
                ],
            }]
        );
    }

    #[test]
    fn duplicate_source_type_identity_is_rejected() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let error = VmStateBlob::from_reflected_objects(
            1,
            vec![
                VmStateTypeIdentity {
                    type_path: type_path.clone(),
                    type_hash: 1,
                },
                VmStateTypeIdentity {
                    type_path: type_path.clone(),
                    type_hash: 2,
                },
            ],
            &[],
        )
        .unwrap_err();

        assert_eq!(
            error,
            VmStateMigrationError::DuplicateSourceTypeIdentity {
                type_path: "game.PlayerState".to_string(),
            }
        );
    }

    #[test]
    fn payload_type_must_be_declared_by_source_type_table() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let error = VmStateBlob::from_reflected_objects(
            1,
            Vec::new(),
            &[VmStateObject {
                type_path: type_path.clone(),
                fields: Vec::new(),
            }],
        )
        .unwrap_err();

        assert_eq!(
            error,
            VmStateMigrationError::MissingSourceTypeIdentity {
                type_path: "game.PlayerState".to_string(),
            }
        );
    }
}
