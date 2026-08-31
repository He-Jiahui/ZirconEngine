use std::fs::File;

use crate::asset::project::{ImportSourceWatchEcho, ProjectGenerationPhase};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetStatusRecord, AssetUri};
use crate::core::CoreError;
use crate::core::resource::ResourceScheme;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_status_record;
use super::ProjectAssetManager;

pub struct ProjectGeneratedSourceReceipt {
    source_uri: AssetUri,
    generation: u64,
    previous_source_hash: Option<blake3::Hash>,
    committed_source_hash: blake3::Hash,
    status: Option<AssetStatusRecord>,
}

impl ProjectGeneratedSourceReceipt {
    pub fn source_uri(&self) -> &AssetUri {
        &self.source_uri
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn previous_source_hash(&self) -> Option<blake3::Hash> {
        self.previous_source_hash
    }

    pub fn committed_source_hash(&self) -> blake3::Hash {
        self.committed_source_hash
    }

    pub fn status(&self) -> Option<&AssetStatusRecord> {
        self.status.as_ref()
    }
}

impl std::fmt::Debug for ProjectGeneratedSourceReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProjectGeneratedSourceReceipt")
            .field("source_uri", &self.source_uri)
            .field("generation", &self.generation)
            .field("previous_source_hash", &self.previous_source_hash)
            .field("committed_source_hash", &self.committed_source_hash)
            .field("status", &self.status)
            .finish()
    }
}

impl ProjectAssetManager {
    /// Publishes generated source bytes through one project generation transaction.
    ///
    /// Import preparation reads the caller-owned snapshot. The source, sidecar, artifacts and
    /// registry remain private until the durable file batch and live resource mutation can commit
    /// together. This method deliberately owns no editor history policy.
    pub fn publish_generated_project_source(
        &self,
        source_uri: AssetUri,
        source_bytes: Vec<u8>,
    ) -> Result<ProjectGeneratedSourceReceipt, CoreError> {
        if source_uri.scheme() != ResourceScheme::Res {
            return Err(asset_error_message(
                "generated project source publication requires a res:// URI",
            ));
        }
        let committed_source_hash = blake3::hash(&source_bytes);
        let (
            expected_generation,
            expected_preparation_epoch,
            mut candidate,
            source_path,
            previous_source_hash,
            previous_source_records,
        ) = {
            let _generation = self.project_generation_read();
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Err(asset_error_message(
                    "generated source publication requires an active project",
                ));
            };
            let source_path = active_project
                .existing_or_primary_project_source_path_for_uri(&source_uri)
                .map_err(asset_error)?;
            let previous_source_hash =
                hash_existing_source(&source_path).map_err(|error| asset_error(error.into()))?;
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                active_project.clone(),
                source_path,
                previous_source_hash,
                active_project.source_resource_records(&source_uri),
            )
        };
        let source_watch_echo = ImportSourceWatchEcho::new(
            source_uri.clone(),
            source_uri.clone(),
            source_path.clone(),
            &source_bytes,
        );
        let mut prepared_generation = candidate
            .prepare_generated_source_generation(&source_uri, &source_path, source_bytes)
            .map_err(asset_error)?;
        let ready_payloads = prepared_generation.take_ready_payloads();
        let prepared = self.prepare_targeted_project_resource_sync(
            &candidate,
            &source_uri,
            source_path,
            &previous_source_records,
            prepared_generation.imported(),
            prepared_generation.affected(),
            ready_payloads,
        );
        let status = candidate
            .registry()
            .get_by_locator(&source_uri)
            .map(build_status_record);
        let committed_generation = candidate.catalog_input_generation().sequence();

        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "generated source publication lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "generated source publication was superseded by a newer project generation",
            ));
        }
        let commit_outcome = self.commit_targeted_project_resource_sync(
            prepared,
            || prepared_generation.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        self.register_transaction_watch_echoes([source_watch_echo]);
        self.publish_project_generation(
            generation,
            status
                .is_some()
                .then(|| {
                    AssetChange::new(
                        if previous_source_hash.is_some() {
                            AssetChangeKind::Modified
                        } else {
                            AssetChangeKind::Added
                        },
                        source_uri.clone(),
                        None,
                    )
                })
                .into_iter()
                .collect(),
        );
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(ProjectGeneratedSourceReceipt {
            source_uri,
            generation: committed_generation,
            previous_source_hash,
            committed_source_hash,
            status,
        })
    }
}

fn hash_existing_source(path: &std::path::Path) -> std::io::Result<Option<blake3::Hash>> {
    let mut source = match File::open(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(&mut source)?;
    Ok(Some(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_source_publication_uses_project_transaction_and_resource_owner() {
        let source = include_str!("generated_source.rs");

        assert!(source.contains("prepare_generated_source_generation("));
        assert!(source.contains("prepare_targeted_project_resource_sync("));
        assert!(source.contains("commit_targeted_project_resource_sync("));
        assert!(source.contains("prepared_generation.commit()"));
        assert!(source.contains("register_transaction_watch_echoes"));
        assert!(source.contains("publish_project_generation("));
        assert!(!source.contains("source_bytes.clone()"));
        assert!(!source.contains("fs::write("));
        assert!(!source.contains("fs::read("));
    }
}
