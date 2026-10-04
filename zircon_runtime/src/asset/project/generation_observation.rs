use std::io;
use std::path::Path;

use super::AssetMetaDocument;

const PROFILE_STREAM: &str = "asset";
const PROFILE_PHASE_CATEGORY: &str = "project_generation.phase";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectGenerationPhase {
    Discovery,
    MetadataProjection,
    Import,
    DependencyProjection,
    RegistryProjection,
    Serialize,
    ResourceProjection,
    ResourceReservation,
    FileCommit,
    ProjectInstall,
    ResourceApply,
    GenerationPublish,
    Recovery,
}

impl ProjectGenerationPhase {
    pub(crate) fn enter(self) -> ProjectGenerationPhaseScope {
        #[cfg(feature = "profiling")]
        let scope = crate::core::runtime::diagnostics::profiling::ProfileScope::enter(
            PROFILE_STREAM,
            PROFILE_PHASE_CATEGORY,
            self.name(),
        );
        #[cfg(not(feature = "profiling"))]
        let _ = self;
        ProjectGenerationPhaseScope {
            #[cfg(feature = "profiling")]
            _scope: scope,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::MetadataProjection => "metadata_projection",
            Self::Import => "import",
            Self::DependencyProjection => "dependency_projection",
            Self::RegistryProjection => "registry_projection",
            Self::Serialize => "serialize",
            Self::ResourceProjection => "resource_projection",
            Self::ResourceReservation => "resource_reservation",
            Self::FileCommit => "file_commit",
            Self::ProjectInstall => "project_install",
            Self::ResourceApply => "resource_apply",
            Self::GenerationPublish => "generation_publish",
            Self::Recovery => "recovery",
        }
    }
}

#[must_use]
pub(crate) struct ProjectGenerationPhaseScope {
    #[cfg(feature = "profiling")]
    _scope: crate::core::runtime::diagnostics::profiling::ProfileScope,
}

#[derive(Debug, Default)]
pub(crate) struct ProjectGenerationObservation {
    source_count: u64,
    compound_member_path_count: u64,
    metadata_document_count: u64,
    existing_metadata_document_count: u64,
    metadata_deserialize_count: u64,
    metadata_deserialize_bytes: u64,
    source_bytes: u64,
    restored_source_count: u64,
    imported_source_count: u64,
    failed_source_count: u64,
    artifact_count: u64,
    artifact_raw_bytes: u64,
    artifact_compressed_bytes: u64,
    artifact_chunk_count: u64,
    artifact_manifest_bytes: u64,
    changed_metadata_count: u64,
    prepared_write_count: u64,
    prepared_write_bytes: u64,
    committed_write_count: u64,
    committed_write_bytes: u64,
    prepare_succeeded: bool,
    commit_succeeded: bool,
}

impl ProjectGenerationObservation {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn load_metadata_document(
        &mut self,
        path: impl AsRef<Path>,
    ) -> io::Result<AssetMetaDocument> {
        let serialized = AssetMetaDocument::read_bounded_text(path)?;
        self.record_metadata_deserialize(serialized.len());
        AssetMetaDocument::from_toml_str(&serialized)
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
    }

    pub(crate) fn record_sources(&mut self, sources: usize, compound_member_paths: usize) {
        self.source_count = usize_to_u64(sources);
        self.compound_member_path_count = usize_to_u64(compound_member_paths);
    }

    pub(crate) fn record_metadata_inventory(&mut self, documents: usize, existing: usize) {
        self.metadata_document_count = usize_to_u64(documents);
        self.existing_metadata_document_count = usize_to_u64(existing);
    }

    pub(crate) fn record_metadata_deserialize(&mut self, bytes: usize) {
        self.metadata_deserialize_count = self.metadata_deserialize_count.saturating_add(1);
        self.metadata_deserialize_bytes = self
            .metadata_deserialize_bytes
            .saturating_add(usize_to_u64(bytes));
    }

    pub(crate) fn record_source_bytes(&mut self, bytes: usize) {
        self.source_bytes = self.source_bytes.saturating_add(usize_to_u64(bytes));
    }

    pub(crate) fn record_restored_source(&mut self) {
        self.restored_source_count = self.restored_source_count.saturating_add(1);
    }

    pub(crate) fn record_imported_source(&mut self) {
        self.imported_source_count = self.imported_source_count.saturating_add(1);
    }

    pub(crate) fn record_failed_source(&mut self) {
        self.failed_source_count = self.failed_source_count.saturating_add(1);
    }

    pub(crate) fn record_artifact(
        &mut self,
        raw_bytes: u64,
        compressed_bytes: u64,
        chunks: usize,
        manifest_bytes: usize,
    ) {
        self.artifact_count = self.artifact_count.saturating_add(1);
        self.artifact_raw_bytes = self.artifact_raw_bytes.saturating_add(raw_bytes);
        self.artifact_compressed_bytes = self
            .artifact_compressed_bytes
            .saturating_add(compressed_bytes);
        self.artifact_chunk_count = self
            .artifact_chunk_count
            .saturating_add(usize_to_u64(chunks));
        self.artifact_manifest_bytes = self
            .artifact_manifest_bytes
            .saturating_add(usize_to_u64(manifest_bytes));
    }

    pub(crate) fn record_changed_metadata(&mut self, documents: usize) {
        self.changed_metadata_count = usize_to_u64(documents);
    }

    pub(crate) fn record_prepared_writes(&mut self, writes: usize, bytes: u64) {
        self.prepared_write_count = usize_to_u64(writes);
        self.prepared_write_bytes = bytes;
    }

    pub(crate) fn mark_prepare_succeeded(&mut self) {
        self.prepare_succeeded = true;
    }

    pub(crate) fn mark_commit_succeeded(&mut self) {
        self.commit_succeeded = true;
        self.committed_write_count = self.prepared_write_count;
        self.committed_write_bytes = self.prepared_write_bytes;
    }

    fn publish(&self) {
        #[cfg(feature = "profiling")]
        {
            use crate::core::runtime::diagnostics::profiling::{
                capture_active, record_counter_batch,
            };

            if !capture_active() {
                return;
            }
            record_counter_batch(
                PROFILE_STREAM,
                &[
                    (
                        "asset.project_generation.source_count",
                        self.source_count as f64,
                    ),
                    (
                        "asset.project_generation.compound_member_path_count",
                        self.compound_member_path_count as f64,
                    ),
                    (
                        "asset.project_generation.metadata_document_count",
                        self.metadata_document_count as f64,
                    ),
                    (
                        "asset.project_generation.existing_metadata_document_count",
                        self.existing_metadata_document_count as f64,
                    ),
                    (
                        "asset.project_generation.metadata_deserialize_count",
                        self.metadata_deserialize_count as f64,
                    ),
                    (
                        "asset.project_generation.metadata_deserialize_bytes",
                        self.metadata_deserialize_bytes as f64,
                    ),
                    (
                        "asset.project_generation.source_bytes",
                        self.source_bytes as f64,
                    ),
                    (
                        "asset.project_generation.restored_source_count",
                        self.restored_source_count as f64,
                    ),
                    (
                        "asset.project_generation.imported_source_count",
                        self.imported_source_count as f64,
                    ),
                    (
                        "asset.project_generation.failed_source_count",
                        self.failed_source_count as f64,
                    ),
                    (
                        "asset.project_generation.artifact_count",
                        self.artifact_count as f64,
                    ),
                    (
                        "asset.project_generation.artifact_raw_bytes",
                        self.artifact_raw_bytes as f64,
                    ),
                    (
                        "asset.project_generation.artifact_compressed_bytes",
                        self.artifact_compressed_bytes as f64,
                    ),
                    (
                        "asset.project_generation.artifact_chunk_count",
                        self.artifact_chunk_count as f64,
                    ),
                    (
                        "asset.project_generation.artifact_manifest_bytes",
                        self.artifact_manifest_bytes as f64,
                    ),
                    (
                        "asset.project_generation.changed_metadata_count",
                        self.changed_metadata_count as f64,
                    ),
                    (
                        "asset.project_generation.prepared_write_count",
                        self.prepared_write_count as f64,
                    ),
                    (
                        "asset.project_generation.prepared_write_bytes",
                        self.prepared_write_bytes as f64,
                    ),
                    (
                        "asset.project_generation.committed_write_count",
                        self.committed_write_count as f64,
                    ),
                    (
                        "asset.project_generation.committed_write_bytes",
                        self.committed_write_bytes as f64,
                    ),
                    (
                        "asset.project_generation.prepare_succeeded",
                        u8::from(self.prepare_succeeded) as f64,
                    ),
                    (
                        "asset.project_generation.commit_succeeded",
                        u8::from(self.commit_succeeded) as f64,
                    ),
                ],
            );
        }
    }
}

impl Drop for ProjectGenerationObservation {
    fn drop(&mut self) {
        self.publish();
    }
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(all(test, feature = "profiling"))]
#[path = "tests/generation_observation.rs"]
mod tests;
