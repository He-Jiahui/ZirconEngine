use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::mutation::{AssetMutationDeleteDisposition, AssetMutationDeletePreflight};
use crate::asset::project::ProjectCatalogInputGeneration;
use crate::asset::{AssetId, AssetImportError, AssetUri, AssetUuid};
use crate::core::resource::{ResourceRecord, ResourceRegistryAssemblyExt};

use super::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};
use super::meta_path_for_source::meta_path_for_source;
use super::relocation::{validate_relocatable_source, verify_meta_preconditions};
use super::ProjectManager;

pub(crate) struct PreparedProjectSourceDeletion {
    journal_directory: PathBuf,
    writes: Vec<PreparedFileWrite>,
    meta_paths: Vec<PathBuf>,
    meta_preconditions: Vec<(PathBuf, Option<crate::asset::project::AssetMetaDocument>)>,
    removed_records: Vec<ResourceRecord>,
    source: AssetUri,
}

impl PreparedProjectSourceDeletion {
    pub(crate) fn removed_records(&self) -> &[ResourceRecord] {
        &self.removed_records
    }

    pub(crate) fn source(&self) -> &AssetUri {
        &self.source
    }

    pub(crate) fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guards = crate::asset::project::lock_meta_document_paths(&self.meta_paths)?;
        verify_meta_preconditions(&self.meta_preconditions)?;
        commit_prepared_files(
            &self.journal_directory,
            self.writes,
            ProjectTransactionFault::None,
        )
    }
}

impl ProjectManager {
    pub(crate) fn prepare_project_source_deletion(
        &mut self,
        target_uuid: AssetUuid,
    ) -> Result<PreparedProjectSourceDeletion, AssetImportError> {
        let preflight = AssetMutationDeletePreflight::evaluate(&self.asset_registry, target_uuid);
        if preflight.disposition() != AssetMutationDeleteDisposition::Ready {
            return Err(delete_preflight_error(&preflight, target_uuid));
        }
        let source = preflight
            .target()
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "project source deletion preflight omitted its ready target: {target_uuid}"
                ))
            })?
            .locator()
            .clone();
        if source == self.manifest.default_scene {
            return Err(AssetImportError::Parse(format!(
                "project source deletion cannot remove the active default scene {source}"
            )));
        }

        let source_path = self.source_path_for_uri(&source)?;
        let meta_path = meta_path_for_source(&source_path);
        let source_bytes = fs::read(&source_path)?;
        let meta_bytes = fs::read(&meta_path)?;
        let source_meta = crate::asset::project::AssetMetaDocument::load(&meta_path)?;
        validate_relocatable_source(&source, &source_meta, &source_bytes)?;

        let removed_records = self.source_resource_records(&source);
        let (asset_registry, removed_uuids) = self
            .asset_registry
            .prepare_source_deletion_generation(&source)?;
        let mut registry = self.registry.begin_staging();
        for record in &removed_records {
            if registry
                .stage_remove_locator(record.primary_locator())
                .is_none()
            {
                return Err(AssetImportError::Parse(format!(
                    "project source deletion could not stage resource {}",
                    record.primary_locator()
                )));
            }
        }
        let removed_ids = removed_uuids
            .iter()
            .copied()
            .map(AssetId::from_asset_uuid)
            .collect::<Vec<_>>();
        let catalog_input_generation = ProjectCatalogInputGeneration::publish_targeted(
            &self.catalog_input_generation,
            self.paths.root(),
            &self.manifest,
            &self.package_assets,
            std::iter::empty(),
            Default::default(),
            removed_ids,
        );
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;

        self.registry = registry.finish();
        self.asset_registry = Arc::new(asset_registry);
        self.catalog_input_generation = catalog_input_generation;
        Ok(PreparedProjectSourceDeletion {
            journal_directory: journal_directory(&self.paths),
            writes: vec![PreparedFileWrite::new(persisted.path, persisted.bytes)
                .retiring_with_expected_digest(
                    source_path,
                    blake3::hash(&source_bytes).to_hex().to_string(),
                )
                .retiring_with_expected_digest(
                    meta_path.clone(),
                    blake3::hash(&meta_bytes).to_hex().to_string(),
                )],
            meta_paths: vec![meta_path.clone()],
            meta_preconditions: vec![(meta_path, Some(source_meta))],
            removed_records,
            source,
        })
    }
}

fn delete_preflight_error(
    preflight: &AssetMutationDeletePreflight,
    target_uuid: AssetUuid,
) -> AssetImportError {
    let detail = match preflight.disposition() {
        AssetMutationDeleteDisposition::Ready => "ready".to_owned(),
        AssetMutationDeleteDisposition::MissingAsset => "asset is missing".to_owned(),
        AssetMutationDeleteDisposition::UnsupportedSubasset => {
            "labeled subassets cannot be deleted independently".to_owned()
        }
        AssetMutationDeleteDisposition::BlockedByReferencers => format!(
            "asset is referenced by {}",
            preflight
                .referencers()
                .iter()
                .map(|asset| asset.locator().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    AssetImportError::Parse(format!(
        "project source deletion preflight rejected {target_uuid}: {detail}"
    ))
}
