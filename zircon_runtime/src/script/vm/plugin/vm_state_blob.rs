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
mod tests {
    use std::{
        collections::BTreeSet,
        hint::black_box,
        time::{Duration, Instant},
    };

    use zircon_runtime_interface::reflect::{ReflectFieldId, ReflectTypePath, ReflectedValue};

    use super::{
        validate_reflected_objects, VmStateBlob, VmStateFieldValue, VmStateObject,
        VmStateTypeIdentity,
    };
    use crate::script::VmStateMigrationError;

    const PERF_SAMPLE_PAIRS: usize = 15;

    fn field_id(field_key: &str) -> ReflectFieldId {
        ReflectFieldId::from_stable_keys("tests.vm-state-field", field_key)
    }

    fn legacy_validate_reflected_objects(
        types: &[VmStateTypeIdentity],
        objects: &[VmStateObject],
    ) -> Result<(), VmStateMigrationError> {
        let mut type_paths = BTreeSet::new();
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
            let mut field_ids = BTreeSet::new();
            for field in &object.fields {
                if !field_ids.insert(field.field_id) {
                    return Err(VmStateMigrationError::DuplicateSourceField {
                        type_path: type_path.to_string(),
                        field: field.field_id.to_string(),
                    });
                }
            }
        }
        Ok(())
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

    fn assert_hash_index_target(label: &str, legacy_p95: Duration, optimized_p95: Duration) {
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= legacy_p95.as_nanos().saturating_mul(80),
            "{label} hash index must reduce P95 by at least 20%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
        );
        assert!(
            optimized_p95 <= Duration::from_millis(20),
            "{label} hash index must remain within the 20 ms release budget: optimized={optimized_p95:?}"
        );
    }

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
                fields: vec![VmStateFieldValue::new(
                    field_id("health"),
                    ReflectedValue::Scalar(75.0),
                )],
            }],
        )
        .unwrap();

        let encoded = state.to_json().unwrap();
        let decoded = VmStateBlob::from_json(&encoded).unwrap();

        assert_eq!(decoded, state);
    }

    #[test]
    fn state_blob_rejects_non_finite_reflected_values() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let error = VmStateBlob::from_reflected_objects(
            2,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 0xA11C_E001,
            }],
            &[VmStateObject {
                type_path,
                fields: vec![VmStateFieldValue::new(
                    field_id("health"),
                    ReflectedValue::Scalar(f32::NAN),
                )],
            }],
        )
        .expect_err("VM state must reject non-finite reflected values before encoding");

        assert!(matches!(
            error,
            VmStateMigrationError::ReflectedValueRejected {
                ref type_path,
                ref field,
                ..
            } if type_path == "game.PlayerState" && field == &field_id("health").to_string()
        ));
    }

    #[test]
    fn v3_state_blob_rejects_legacy_field_name_payloads() {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        let payload = serde_json::to_vec(&serde_json::json!([{
            "type_path": type_path,
            "fields": [{
                "field_name": "health",
                "value": {"kind": "Scalar", "value": 75.0}
            }]
        }]))
        .unwrap();
        let blob = VmStateBlob {
            schema_version: super::VM_STATE_SCHEMA_VERSION_V3,
            types: vec![VmStateTypeIdentity {
                type_path: ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap(),
                type_hash: 1,
            }],
            payload,
        };

        assert!(matches!(
            blob.reflected_objects(),
            Err(VmStateMigrationError::PayloadDecode { .. })
        ));
    }

    #[test]
    fn optimization_wave_20260825vw_runtime129_validation_keeps_input_error_precedence() {
        let known_type = ReflectTypePath::new("game.KnownState", "KnownState").unwrap();
        let missing_type = ReflectTypePath::new("game.MissingState", "MissingState").unwrap();
        let types = vec![VmStateTypeIdentity {
            type_path: known_type.clone(),
            type_hash: 1,
        }];
        let objects = vec![
            VmStateObject {
                type_path: known_type,
                fields: vec![
                    VmStateFieldValue::new(field_id("value"), ReflectedValue::Scalar(1.0)),
                    VmStateFieldValue::new(field_id("value"), ReflectedValue::Scalar(2.0)),
                ],
            },
            VmStateObject {
                type_path: missing_type,
                fields: Vec::new(),
            },
        ];

        assert_eq!(
            validate_reflected_objects(&types, &objects),
            Err(VmStateMigrationError::DuplicateSourceField {
                type_path: "game.KnownState".to_string(),
                field: field_id("value").to_string(),
            })
        );
    }

    #[test]
    fn optimization_wave_20260825vw_runtime129_validation_uses_hash_indexes() {
        let production = include_str!("vm_state_blob.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("HashSet::with_capacity(types.len())"));
        assert!(production.contains("HashSet::with_capacity(object.fields.len())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "managed Runtime129 performance evidence"]
    fn optimization_wave_20260825vw_runtime129_validation_hash_index_evidence() {
        const TYPE_COUNT: usize = 16_384;
        const FIELD_COUNT: usize = 16_384;

        let types = (0..TYPE_COUNT)
            .map(|index| VmStateTypeIdentity {
                type_path: ReflectTypePath::new(
                    format!("game.State{index:05}"),
                    format!("State{index:05}"),
                )
                .unwrap(),
                type_hash: index as u32,
            })
            .collect::<Vec<_>>();
        let objects = (0..TYPE_COUNT)
            .map(|index| VmStateObject {
                type_path: types[(index * 8_191) % TYPE_COUNT].type_path.clone(),
                fields: Vec::new(),
            })
            .collect::<Vec<_>>();
        let legacy_type_validation = || {
            legacy_validate_reflected_objects(black_box(&types), black_box(&objects)).unwrap();
            types.len() + objects.len()
        };
        let optimized_type_validation = || {
            validate_reflected_objects(black_box(&types), black_box(&objects)).unwrap();
            types.len() + objects.len()
        };
        assert_eq!(legacy_type_validation(), optimized_type_validation());
        black_box(legacy_type_validation());
        black_box(optimized_type_validation());
        let (legacy_type_samples, optimized_type_samples) =
            measure_pairs(legacy_type_validation, optimized_type_validation);

        let field_type = ReflectTypePath::new("game.FieldState", "FieldState").unwrap();
        let field_types = vec![VmStateTypeIdentity {
            type_path: field_type.clone(),
            type_hash: 1,
        }];
        let field_objects = vec![VmStateObject {
            type_path: field_type,
            fields: (0..FIELD_COUNT)
                .map(|index| {
                    VmStateFieldValue::new(
                        field_id(&format!("field_{index:05}")),
                        ReflectedValue::Scalar(index as f32),
                    )
                })
                .collect(),
        }];
        let legacy_field_validation = || {
            legacy_validate_reflected_objects(black_box(&field_types), black_box(&field_objects))
                .unwrap();
            FIELD_COUNT
        };
        let optimized_field_validation = || {
            validate_reflected_objects(black_box(&field_types), black_box(&field_objects)).unwrap();
            FIELD_COUNT
        };
        assert_eq!(legacy_field_validation(), optimized_field_validation());
        black_box(legacy_field_validation());
        black_box(optimized_field_validation());
        let (legacy_field_samples, optimized_field_samples) =
            measure_pairs(legacy_field_validation, optimized_field_validation);

        let legacy_type_p95 = nearest_rank(&legacy_type_samples, 95);
        let optimized_type_p95 = nearest_rank(&optimized_type_samples, 95);
        let legacy_field_p95 = nearest_rank(&legacy_field_samples, 95);
        let optimized_field_p95 = nearest_rank(&optimized_field_samples, 95);
        eprintln!(
            "RUNTIME129_VM_STATE_VALIDATION_HASH_INDEX_BENCH_V1 type_identities={TYPE_COUNT} objects={TYPE_COUNT} fields={FIELD_COUNT} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even type_legacy_p50_ns={} type_legacy_p95_ns={} type_optimized_p50_ns={} type_optimized_p95_ns={} field_legacy_p50_ns={} field_legacy_p95_ns={} field_optimized_p50_ns={} field_optimized_p95_ns={} type_legacy_ns={} type_optimized_ns={} field_legacy_ns={} field_optimized_ns={}",
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
        assert_hash_index_target("type-table validation", legacy_type_p95, optimized_type_p95);
        assert_hash_index_target(
            "object-field validation",
            legacy_field_p95,
            optimized_field_p95,
        );
    }
}
