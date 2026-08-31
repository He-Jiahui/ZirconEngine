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
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        hint::black_box,
        time::{Duration, Instant},
    };

    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectSerializationStrategy,
        ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
    };

    use super::{
        index_target_types, migrate_object, migrate_vm_state_blob, VmStateMigrationError,
        VmStateSchema, VmStateTypeSchema,
    };
    use crate::script::{VmStateBlob, VmStateFieldValue, VmStateObject, VmStateTypeIdentity};

    const PERF_SAMPLE_PAIRS: usize = 15;

    fn state_field_id(field_key: &str) -> ReflectFieldId {
        ReflectFieldId::from_stable_keys("tests.vm-state-field", field_key)
    }

    fn state_field(
        name: impl Into<String>,
        value_type_path: impl Into<String>,
        editor_hint: ReflectEditorHint,
    ) -> ReflectFieldInfo {
        let name = name.into();
        ReflectFieldInfo::new(
            state_field_id(&name),
            name.clone(),
            value_type_path,
            editor_hint,
        )
    }

    fn legacy_index_target_types(
        target: &VmStateSchema,
    ) -> Result<BTreeMap<&str, &VmStateTypeSchema>, VmStateMigrationError> {
        let mut target_types = BTreeMap::new();
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
        }
        Ok(target_types)
    }

    fn legacy_validate_target_fields(
        target: &VmStateTypeSchema,
    ) -> Result<BTreeSet<ReflectFieldId>, VmStateMigrationError> {
        let mut ids = BTreeSet::new();
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

    fn legacy_migrate_object(
        object: VmStateObject,
        target: &VmStateTypeSchema,
    ) -> Result<VmStateObject, VmStateMigrationError> {
        let mut source_fields = BTreeMap::new();
        for field in object.fields {
            source_fields.insert(field.field_id, field.value);
        }

        legacy_validate_target_fields(target)?;
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

    fn measure_pairs<L, O>(mut legacy: L, mut optimized: O) -> (Vec<Duration>, Vec<Duration>)
    where
        L: FnMut() -> usize,
        O: FnMut() -> usize,
    {
        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                black_box(legacy());
                legacy_samples.push(started.elapsed());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                black_box(optimized());
                optimized_samples.push(started.elapsed());
            };
            if pair % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn duration_csv(samples: &[Duration]) -> String {
        samples
            .iter()
            .map(|sample| sample.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn assert_hash_index_target(
        label: &str,
        legacy_p95: Duration,
        optimized_p95: Duration,
        maximum_ratio_percent: u128,
        maximum_elapsed: Duration,
    ) {
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= legacy_p95
                    .as_nanos()
                    .saturating_mul(maximum_ratio_percent),
            "{label} hash index exceeds the P95 ratio target: legacy={legacy_p95:?}, optimized={optimized_p95:?}, maximum_ratio_percent={maximum_ratio_percent}"
        );
        assert!(
            optimized_p95 <= maximum_elapsed,
            "{label} hash index exceeds the release budget: optimized={optimized_p95:?}, maximum={maximum_elapsed:?}"
        );
    }

    #[test]
    fn vm_schema_rejects_non_finite_default_values_before_encoding() {
        let schema = VmStateSchema {
            schema_version: 2,
            types: vec![VmStateTypeSchema {
                registration: ReflectTypeRegistration::new(
                    ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap(),
                    "Player State",
                    ReflectTypeInfo::struct_with_fields(vec![state_field(
                        "health",
                        "Scalar",
                        ReflectEditorHint::Scalar,
                    )
                    .with_default_value(ReflectedValue::Scalar(f32::NAN))]),
                    ReflectSerializationStrategy::Value,
                ),
                type_hash: 1,
            }],
        };

        assert!(matches!(
            schema
                .to_json()
                .expect_err("VM schema defaults must pass reflection value admission"),
            VmStateMigrationError::ReflectedValueRejected {
                ref type_path,
                ref field,
                ..
            } if type_path == "game.PlayerState" && field == "health"
        ));
    }

    #[test]
    fn vm_schema_rejects_legacy_field_rename_maps() {
        let schema = VmStateSchema {
            schema_version: 3,
            types: vec![VmStateTypeSchema {
                registration: ReflectTypeRegistration::new(
                    ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap(),
                    "Player State",
                    ReflectTypeInfo::struct_with_fields(vec![state_field(
                        "health",
                        "f64",
                        ReflectEditorHint::Scalar,
                    )]),
                    ReflectSerializationStrategy::Value,
                ),
                type_hash: 1,
            }],
        };
        let mut encoded = serde_json::to_value(schema).unwrap();
        encoded["types"][0]["renames"] = serde_json::json!([{
            "from": "old_health",
            "to": "health"
        }]);

        assert!(matches!(
            VmStateSchema::from_json(&encoded.to_string()),
            Err(VmStateMigrationError::SchemaDecode { .. })
        ));
    }

    #[test]
    fn stable_field_id_survives_current_name_change_without_migration_map() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let source_health = ReflectFieldInfo::new(
            state_field_id("health"),
            "old_health",
            "f64",
            ReflectEditorHint::Scalar,
        );
        assert_ne!(source_health.name, "health");
        let source = VmStateBlob::from_reflected_objects(
            1,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 1,
            }],
            &[VmStateObject {
                type_path: type_path.clone(),
                fields: vec![
                    VmStateFieldValue::new(source_health.id, ReflectedValue::Scalar(75.0)),
                    VmStateFieldValue::new(
                        state_field_id("name"),
                        ReflectedValue::String("Ada".to_string()),
                    ),
                    VmStateFieldValue::new(state_field_id("removed"), ReflectedValue::Bool(true)),
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
                        state_field("health", "f64", ReflectEditorHint::Scalar),
                        state_field("max_health", "f64", ReflectEditorHint::Scalar)
                            .with_default_value(ReflectedValue::Scalar(100.0)),
                        state_field("name", "String", ReflectEditorHint::String),
                    ]),
                    ReflectSerializationStrategy::Value,
                ),
                type_hash: 2,
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
                    VmStateFieldValue::new(state_field_id("health"), ReflectedValue::Scalar(75.0),),
                    VmStateFieldValue::new(
                        state_field_id("max_health"),
                        ReflectedValue::Scalar(100.0),
                    ),
                    VmStateFieldValue::new(
                        state_field_id("name"),
                        ReflectedValue::String("Ada".to_string()),
                    ),
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

    #[test]
    fn optimization_wave_20260825vw_runtime129_migration_uses_hash_indexes() {
        let production = include_str!("state_migration.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();

        assert!(production.contains("use std::collections::{HashMap, HashSet};"));
        assert!(production.contains("HashMap::with_capacity(target.types.len())"));
        assert!(production.contains("HashMap::with_capacity(object.fields.len())"));
        assert!(production
            .contains("HashSet::with_capacity(target.registration.type_info.fields.len())"));
        assert!(!production.contains("VmStateFieldRename"));
        assert!(!production.contains("BTreeMap"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "managed Runtime129 performance evidence"]
    fn optimization_wave_20260825vw_runtime129_migration_hash_index_evidence() {
        const TYPE_COUNT: usize = 8_192;
        const FIELD_COUNT: usize = 4_096;

        let target = VmStateSchema {
            schema_version: 2,
            types: (0..TYPE_COUNT)
                .map(|index| VmStateTypeSchema {
                    registration: ReflectTypeRegistration::new(
                        ReflectTypePath::new(
                            format!("game.State{index:05}"),
                            format!("State{index:05}"),
                        )
                        .unwrap(),
                        format!("State {index:05}"),
                        ReflectTypeInfo::struct_with_fields(Vec::new()),
                        ReflectSerializationStrategy::Value,
                    ),
                    type_hash: index as u32,
                })
                .collect(),
        };
        let target_paths = target
            .types
            .iter()
            .map(|schema| schema.registration.type_path.type_path())
            .collect::<Vec<_>>();
        let legacy_type_index = || {
            let index = legacy_index_target_types(black_box(&target)).unwrap();
            (0..TYPE_COUNT).fold(index.len(), |sum, probe| {
                let type_path = target_paths[(probe * 4_093) % TYPE_COUNT];
                sum.wrapping_add(index.get(type_path).unwrap().type_hash as usize)
            })
        };
        let optimized_type_index = || {
            let index = index_target_types(black_box(&target)).unwrap();
            (0..TYPE_COUNT).fold(index.len(), |sum, probe| {
                let type_path = target_paths[(probe * 4_093) % TYPE_COUNT];
                sum.wrapping_add(index.get(type_path).unwrap().type_hash as usize)
            })
        };
        assert_eq!(legacy_type_index(), optimized_type_index());
        black_box(legacy_type_index());
        black_box(optimized_type_index());
        let (legacy_type_samples, optimized_type_samples) =
            measure_pairs(legacy_type_index, optimized_type_index);

        let field_names = (0..FIELD_COUNT)
            .map(|index| format!("field_{index:05}"))
            .collect::<Vec<_>>();
        let migrated_type_path =
            ReflectTypePath::new("game.MigratedState", "MigratedState").unwrap();
        let migrated_type = VmStateTypeSchema {
            registration: ReflectTypeRegistration::new(
                migrated_type_path.clone(),
                "Migrated State",
                ReflectTypeInfo::struct_with_fields(
                    field_names
                        .iter()
                        .map(|name| state_field(name.clone(), "f64", ReflectEditorHint::Scalar))
                        .collect(),
                ),
                ReflectSerializationStrategy::Value,
            ),
            type_hash: 2,
        };
        let source_object = VmStateObject {
            type_path: migrated_type_path,
            fields: (0..FIELD_COUNT)
                .map(|probe| {
                    let index = (probe * 4_093) % FIELD_COUNT;
                    VmStateFieldValue::new(
                        state_field_id(&field_names[index]),
                        ReflectedValue::Scalar(index as f32),
                    )
                })
                .collect(),
        };
        assert_eq!(
            legacy_migrate_object(source_object.clone(), &migrated_type).unwrap(),
            migrate_object(source_object.clone(), &migrated_type).unwrap()
        );
        let legacy_field_migration = || {
            legacy_migrate_object(black_box(source_object.clone()), black_box(&migrated_type))
                .unwrap()
                .fields
                .len()
        };
        let optimized_field_migration = || {
            migrate_object(black_box(source_object.clone()), black_box(&migrated_type))
                .unwrap()
                .fields
                .len()
        };
        assert_eq!(legacy_field_migration(), optimized_field_migration());
        black_box(legacy_field_migration());
        black_box(optimized_field_migration());
        let (legacy_field_samples, optimized_field_samples) =
            measure_pairs(legacy_field_migration, optimized_field_migration);

        let legacy_type_p95 = nearest_rank(&legacy_type_samples, 95);
        let optimized_type_p95 = nearest_rank(&optimized_type_samples, 95);
        let legacy_field_p95 = nearest_rank(&legacy_field_samples, 95);
        let optimized_field_p95 = nearest_rank(&optimized_field_samples, 95);
        eprintln!(
            "RUNTIME129_VM_STATE_MIGRATION_HASH_INDEX_BENCH_V2 target_types={TYPE_COUNT} stable_id_fields={FIELD_COUNT} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even type_legacy_p50_ns={} type_legacy_p95_ns={} type_optimized_p50_ns={} type_optimized_p95_ns={} field_legacy_p50_ns={} field_legacy_p95_ns={} field_optimized_p50_ns={} field_optimized_p95_ns={} type_legacy_ns={} type_optimized_ns={} field_legacy_ns={} field_optimized_ns={}",
            nearest_rank(&legacy_type_samples, 50).as_nanos(),
            legacy_type_p95.as_nanos(),
            nearest_rank(&optimized_type_samples, 50).as_nanos(),
            optimized_type_p95.as_nanos(),
            nearest_rank(&legacy_field_samples, 50).as_nanos(),
            legacy_field_p95.as_nanos(),
            nearest_rank(&optimized_field_samples, 50).as_nanos(),
            optimized_field_p95.as_nanos(),
            duration_csv(&legacy_type_samples),
            duration_csv(&optimized_type_samples),
            duration_csv(&legacy_field_samples),
            duration_csv(&optimized_field_samples),
        );
        assert_hash_index_target(
            "target-type indexing",
            legacy_type_p95,
            optimized_type_p95,
            80,
            Duration::from_millis(20),
        );
        assert_hash_index_target(
            "field migration",
            legacy_field_p95,
            optimized_field_p95,
            90,
            Duration::from_millis(50),
        );
    }
}
