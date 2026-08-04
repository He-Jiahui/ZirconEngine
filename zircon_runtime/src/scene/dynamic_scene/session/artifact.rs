use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self as std_io, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use zircon_runtime_interface::serialization::write_canonical_text_to;
use zircon_runtime_interface::serialization::CanonicalTextWriteError;

use super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionArchivePayload, RuntimeSessionArchiveStatistics, RuntimeSessionSlot,
    RuntimeSessionSlotSummary,
};

pub const MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES: usize = 512 * 1024 * 1024;

pub(super) enum RuntimeSessionArchiveSealState {
    Open,
    Sealed(RuntimeSessionArchiveArtifact),
    Rejected(RuntimeSessionArchiveSealFailure),
}

#[derive(Clone)]
pub(super) enum RuntimeSessionArchiveSealFailure {
    ArtifactTooLarge {
        estimated_bytes: usize,
        limit_bytes: usize,
    },
    NonFinite {
        value: f64,
    },
    PayloadValidation {
        reason: String,
    },
    PayloadEncode {
        reason: String,
    },
    CanonicalOutputTooLarge {
        max: usize,
        found: usize,
    },
    Io {
        operation: &'static str,
        kind: std_io::ErrorKind,
        reason: String,
    },
}

impl RuntimeSessionArchiveSealFailure {
    fn from_canonical(error: CanonicalTextWriteError) -> Self {
        match error {
            CanonicalTextWriteError::NonFinite { value } => Self::NonFinite { value },
            CanonicalTextWriteError::PayloadValidation { reason } => {
                Self::PayloadValidation { reason }
            }
            CanonicalTextWriteError::PayloadEncode { reason } => Self::PayloadEncode { reason },
            CanonicalTextWriteError::OutputTooLarge { max, found } => {
                Self::CanonicalOutputTooLarge { max, found }
            }
            CanonicalTextWriteError::Io { operation, source } => Self::Io {
                operation,
                kind: source.kind(),
                reason: source.to_string(),
            },
        }
    }

    fn to_error(&self) -> RuntimeSessionArchiveError {
        match self {
            Self::ArtifactTooLarge {
                estimated_bytes,
                limit_bytes,
            } => RuntimeSessionArchiveError::ArtifactTooLarge {
                estimated_bytes: *estimated_bytes,
                limit_bytes: *limit_bytes,
            },
            Self::NonFinite { value } => {
                CanonicalTextWriteError::NonFinite { value: *value }.into()
            }
            Self::PayloadValidation { reason } => CanonicalTextWriteError::PayloadValidation {
                reason: reason.clone(),
            }
            .into(),
            Self::PayloadEncode { reason } => CanonicalTextWriteError::PayloadEncode {
                reason: reason.clone(),
            }
            .into(),
            Self::CanonicalOutputTooLarge { max, found } => {
                CanonicalTextWriteError::OutputTooLarge {
                    max: *max,
                    found: *found,
                }
                .into()
            }
            Self::Io {
                operation,
                kind,
                reason,
            } => CanonicalTextWriteError::Io {
                operation,
                source: std_io::Error::new(*kind, reason.clone()),
            }
            .into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveArtifactDiagnostics {
    pub capture_count: usize,
    pub normalize_count: usize,
    pub validate_count: usize,
    pub serialize_count: usize,
    pub internal_json_roundtrip_count: usize,
}

#[derive(Clone, Debug)]
pub struct RuntimeSessionArchiveArtifact {
    generation: u64,
    lineage: u64,
    revision: u64,
    payload: Arc<RuntimeSessionArchivePayload>,
    manifest: Arc<RuntimeSessionArchiveManifest>,
    statistics: RuntimeSessionArchiveStatistics,
    slot_index: Arc<RuntimeSessionSlotIndex>,
    serialized_bytes: Arc<[u8]>,
    counters: Arc<super::archive::RuntimeSessionArchiveStageCounters>,
}

impl RuntimeSessionArchive {
    pub fn sealed_artifact(
        &self,
    ) -> Result<RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError> {
        self.sealed_artifact_with_limit(MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES)
    }

    pub fn artifact_diagnostics(&self) -> RuntimeSessionArchiveArtifactDiagnostics {
        diagnostics_from(&self.state.counters)
    }

    fn sealed_artifact_with_limit(
        &self,
        limit_bytes: usize,
    ) -> Result<RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError> {
        let mut sealed = self
            .state
            .sealed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*sealed {
            RuntimeSessionArchiveSealState::Sealed(artifact) => return Ok(artifact.clone()),
            RuntimeSessionArchiveSealState::Rejected(failure) => return Err(failure.to_error()),
            RuntimeSessionArchiveSealState::Open => {}
        }

        if !self.state.counters.validated.load(Ordering::Acquire) {
            if let Err(error) = self.ensure_supported() {
                let failure = RuntimeSessionArchiveSealFailure::PayloadValidation {
                    reason: error.to_string(),
                };
                let returned = failure.to_error();
                *sealed = RuntimeSessionArchiveSealState::Rejected(failure);
                return Err(returned);
            }
            self.record_validated();
        }
        let payload = self.payload_arc();
        let mut serialized_bytes = BoundedArchiveBytes::new(limit_bytes);
        self.state
            .counters
            .serialize_count
            .fetch_add(1, Ordering::AcqRel);
        if let Err(error) = write_canonical_text_to(&*payload, &mut serialized_bytes) {
            let failure = if let Some(estimated_bytes) = serialized_bytes.overflow_at() {
                RuntimeSessionArchiveSealFailure::ArtifactTooLarge {
                    estimated_bytes,
                    limit_bytes,
                }
            } else {
                RuntimeSessionArchiveSealFailure::from_canonical(error)
            };
            let returned = failure.to_error();
            *sealed = RuntimeSessionArchiveSealState::Rejected(failure);
            return Err(returned);
        }
        // Do not allocate derived indexes before the authoritative byte bound accepts the payload.
        let manifest = Arc::new(build_manifest(&payload));
        let statistics = build_statistics(&payload);
        let slot_index = Arc::new(build_slot_index(&manifest));
        let artifact = RuntimeSessionArchiveArtifact {
            generation: self.generation(),
            lineage: self.lineage(),
            revision: self.revision(),
            payload,
            manifest,
            statistics,
            slot_index,
            serialized_bytes: serialized_bytes.bytes.into(),
            counters: Arc::clone(&self.state.counters),
        };
        *sealed = RuntimeSessionArchiveSealState::Sealed(artifact.clone());
        Ok(artifact)
    }

    #[cfg(test)]
    pub(crate) fn sealed_artifact_with_limit_for_test(
        &self,
        limit_bytes: usize,
    ) -> Result<RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError> {
        self.sealed_artifact_with_limit(limit_bytes)
    }
}

#[derive(Default, Debug)]
struct RuntimeSessionSlotIndex {
    buckets: HashMap<u64, RuntimeSessionSlotIndexBucket>,
    slot_count: usize,
}

#[derive(Debug)]
enum RuntimeSessionSlotIndexBucket {
    One(usize),
    Collision(Vec<usize>),
}

impl RuntimeSessionSlotIndex {
    fn insert(&mut self, slot_id: &str, index: usize) {
        let hash = slot_id_hash(slot_id);
        match self.buckets.entry(hash) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(RuntimeSessionSlotIndexBucket::One(index));
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let bucket = entry.get_mut();
                match bucket {
                    RuntimeSessionSlotIndexBucket::One(existing) => {
                        let first = *existing;
                        *bucket = RuntimeSessionSlotIndexBucket::Collision(vec![first, index]);
                    }
                    RuntimeSessionSlotIndexBucket::Collision(indices) => indices.push(index),
                }
            }
        }
        self.slot_count = self.slot_count.saturating_add(1);
    }

    fn get(&self, slot_id: &str, manifest: &RuntimeSessionArchiveManifest) -> Option<usize> {
        let matches = |index: &usize| {
            manifest
                .slots
                .get(*index)
                .is_some_and(|slot| slot.slot_id == slot_id)
        };
        match self.buckets.get(&slot_id_hash(slot_id))? {
            RuntimeSessionSlotIndexBucket::One(index) => matches(index).then_some(*index),
            RuntimeSessionSlotIndexBucket::Collision(indices) => {
                indices.iter().find(|index| matches(index)).copied()
            }
        }
    }

    fn len(&self) -> usize {
        self.slot_count
    }
}

fn build_slot_index(manifest: &RuntimeSessionArchiveManifest) -> RuntimeSessionSlotIndex {
    let mut index = RuntimeSessionSlotIndex::default();
    for (slot_index, slot) in manifest.slots.iter().enumerate() {
        index.insert(&slot.slot_id, slot_index);
    }
    index
}

fn slot_id_hash(slot_id: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    slot_id.hash(&mut hasher);
    hasher.finish()
}

struct BoundedArchiveBytes {
    bytes: Vec<u8>,
    budget: ArchiveByteBudget,
}

impl BoundedArchiveBytes {
    fn new(limit_bytes: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(limit_bytes.min(64 * 1024)),
            budget: ArchiveByteBudget::new(limit_bytes),
        }
    }

    fn overflow_at(&self) -> Option<usize> {
        self.budget.overflow_at
    }
}

impl Write for BoundedArchiveBytes {
    fn write(&mut self, buffer: &[u8]) -> std_io::Result<usize> {
        self.budget.write_all(buffer)?;
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std_io::Result<()> {
        Ok(())
    }
}

struct ArchiveByteBudget {
    written_bytes: usize,
    limit_bytes: usize,
    overflow_at: Option<usize>,
}

impl ArchiveByteBudget {
    fn new(limit_bytes: usize) -> Self {
        Self {
            written_bytes: 0,
            limit_bytes,
            overflow_at: None,
        }
    }
}

impl Write for ArchiveByteBudget {
    fn write(&mut self, buffer: &[u8]) -> std_io::Result<usize> {
        let projected = self.written_bytes.saturating_add(buffer.len());
        if projected > self.limit_bytes {
            self.overflow_at = Some(projected);
            return Err(std_io::Error::new(
                std_io::ErrorKind::OutOfMemory,
                "runtime session archive artifact byte limit exceeded",
            ));
        }
        self.written_bytes = projected;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std_io::Result<()> {
        Ok(())
    }
}

impl RuntimeSessionArchiveArtifact {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn lineage(&self) -> u64 {
        self.lineage
    }

    pub fn manifest(&self) -> &RuntimeSessionArchiveManifest {
        &self.manifest
    }

    pub fn statistics(&self) -> &RuntimeSessionArchiveStatistics {
        &self.statistics
    }

    pub fn slot_summary(&self, slot_id: &str) -> Option<&RuntimeSessionSlotSummary> {
        self.slot_index
            .get(slot_id, &self.manifest)
            .and_then(|index| self.manifest.slots.get(index))
    }

    pub fn serialized_bytes(&self) -> &[u8] {
        &self.serialized_bytes
    }

    pub fn shares_payload_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.payload, &other.payload)
            && Arc::ptr_eq(&self.serialized_bytes, &other.serialized_bytes)
    }

    pub fn diagnostics(&self) -> RuntimeSessionArchiveArtifactDiagnostics {
        diagnostics_from(&self.counters)
    }

    pub fn save_to_path_atomically(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        io::save_artifact_to_path_atomically(self, path)
    }

    pub(super) fn write_to<W: Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, RuntimeSessionArchiveError> {
        sink.write_all(&self.serialized_bytes)?;
        Ok(self.serialized_bytes.len())
    }
}

fn diagnostics_from(
    counters: &super::archive::RuntimeSessionArchiveStageCounters,
) -> RuntimeSessionArchiveArtifactDiagnostics {
    RuntimeSessionArchiveArtifactDiagnostics {
        capture_count: counters.capture_count.load(Ordering::Acquire),
        normalize_count: counters.normalize_count.load(Ordering::Acquire),
        validate_count: counters.validate_count.load(Ordering::Acquire),
        serialize_count: counters.serialize_count.load(Ordering::Acquire),
        internal_json_roundtrip_count: counters
            .internal_json_roundtrip_count
            .load(Ordering::Acquire),
    }
}

fn build_manifest(payload: &RuntimeSessionArchivePayload) -> RuntimeSessionArchiveManifest {
    RuntimeSessionArchiveManifest {
        format_version: payload.format_version,
        slots: Arc::new(
            payload
                .slots
                .iter()
                .map(RuntimeSessionSlot::summary)
                .collect(),
        ),
    }
}

fn build_statistics(payload: &RuntimeSessionArchivePayload) -> RuntimeSessionArchiveStatistics {
    let mut statistics = RuntimeSessionArchiveStatistics {
        format_version: payload.format_version,
        slot_count: payload.slots.len(),
        ..Default::default()
    };
    for slot in &payload.slots {
        let entity_count = slot.scene.entities.len();
        let resource_count = slot.scene.resources.len();
        record_slot_counts(&mut statistics, entity_count, resource_count);
        if let Some(updated_at) = slot.metadata.updated_at_unix_millis {
            statistics.earliest_updated_at_unix_millis = Some(
                statistics
                    .earliest_updated_at_unix_millis
                    .map_or(updated_at, |current| current.min(updated_at)),
            );
            statistics.latest_updated_at_unix_millis = Some(
                statistics
                    .latest_updated_at_unix_millis
                    .map_or(updated_at, |current| current.max(updated_at)),
            );
        } else {
            statistics.untimed_slot_count = statistics.untimed_slot_count.saturating_add(1);
        }
    }
    statistics
}

fn record_slot_counts(
    statistics: &mut RuntimeSessionArchiveStatistics,
    entity_count: usize,
    resource_count: usize,
) {
    statistics.total_entity_count = statistics.total_entity_count.saturating_add(entity_count);
    statistics.total_resource_count = statistics
        .total_resource_count
        .saturating_add(resource_count);
    statistics.max_slot_entity_count = statistics.max_slot_entity_count.max(entity_count);
    statistics.max_slot_resource_count = statistics.max_slot_resource_count.max(resource_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{
        dynamic_scene::{session::RuntimeSessionMetadata, DynamicResource, DynamicScene},
        NodeKind, World,
    };

    #[test]
    fn runtime_session_archive_payload_limit_matrix_stops_stream_writes_at_bound() {
        let chunk = [0u8; 64 * 1024];
        for mebibytes in [1usize, 64, 512] {
            let limit = mebibytes * 1024 * 1024;
            let mut budget = ArchiveByteBudget::new(limit);
            for _ in 0..limit / chunk.len() {
                budget
                    .write_all(&chunk)
                    .expect("stream should accept bytes through its exact limit");
            }
            assert_eq!(budget.written_bytes, limit);
            assert!(budget.write_all(&[0]).is_err());
            assert_eq!(budget.overflow_at, Some(limit.saturating_add(1)));
        }
        assert_eq!(
            MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
            512 * 1024 * 1024
        );
    }

    #[test]
    fn runtime_session_archive_slot_and_entity_scale_matrix_builds_linear_indexes() {
        let mut source = World::empty();
        source.spawn_node(NodeKind::Mesh);
        let mut scene =
            DynamicScene::from_world(&source).expect("source scene should capture one real entity");
        scene.resources.push(DynamicResource::new(
            "zircon_runtime::tests::ArchiveScaleResource",
            Vec::new(),
        ));
        let template = RuntimeSessionSlot {
            slot_id: "template".to_owned(),
            metadata: RuntimeSessionMetadata::default(),
            scene,
        };

        for count in [1usize, 1_000, 100_000] {
            let payload = RuntimeSessionArchivePayload::new(
                super::super::RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
                (0..count)
                    .map(|index| {
                        let mut slot = template.clone();
                        slot.slot_id = format!("slot-{index:06}");
                        slot
                    })
                    .collect(),
            );
            let manifest = build_manifest(&payload);
            let index = build_slot_index(&manifest);
            assert_eq!(manifest.slot_count(), count);
            assert_eq!(index.len(), count);

            let statistics = build_statistics(&payload);
            assert_eq!(statistics.total_entity_count, count);
            assert_eq!(statistics.total_resource_count, count);
            assert_eq!(statistics.max_slot_entity_count, 1);
            assert_eq!(statistics.max_slot_resource_count, 1);
        }
    }
}
