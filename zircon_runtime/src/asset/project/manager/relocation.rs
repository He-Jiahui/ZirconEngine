use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::mutation::{
    AssetMutationRelocationDisposition, AssetMutationRelocationPreflight,
};
use crate::asset::project::{
    AssetMetaDocument, AssetSourceUnit, ProjectCatalogInputGeneration, ProjectCatalogInputSource,
};
use crate::asset::{AssetId, AssetImportError, AssetReference, AssetUri, AssetUuid};
use crate::core::resource::{ResourceRecord, ResourceRegistryAssemblyExt};

use super::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};
use super::hash_bytes::hash_bytes;
use super::meta_path_for_source::meta_path_for_source;
use super::scan_and_import::refresh_runtime_dependency_closure;
use super::ProjectManager;

pub(crate) struct PreparedProjectSourceRelocation {
    journal_directory: PathBuf,
    writes: Vec<PreparedFileWrite>,
    meta_paths: Vec<PathBuf>,
    meta_preconditions: Vec<(PathBuf, Option<AssetMetaDocument>)>,
    updated: Vec<ResourceRecord>,
    source: AssetUri,
    target: AssetUri,
    target_path: PathBuf,
    locator_moves: Vec<(AssetUri, AssetUri)>,
}

impl PreparedProjectSourceRelocation {
    pub(crate) fn updated_records(&self) -> &[ResourceRecord] {
        &self.updated
    }

    pub(crate) fn source(&self) -> &AssetUri {
        &self.source
    }

    pub(crate) fn target(&self) -> &AssetUri {
        &self.target
    }

    pub(crate) fn target_path(&self) -> &Path {
        &self.target_path
    }

    pub(crate) fn locator_moves(&self) -> &[(AssetUri, AssetUri)] {
        &self.locator_moves
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

    #[cfg(test)]
    fn commit_with_fault(
        self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guards = crate::asset::project::lock_meta_document_paths(&self.meta_paths)?;
        verify_meta_preconditions(&self.meta_preconditions)?;
        commit_prepared_files(&self.journal_directory, self.writes, fault)
    }
}

impl ProjectManager {
    #[cfg(test)]
    pub(crate) fn relocate_project_source_for_test(
        &mut self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.relocate_project_source_with_fault(source_uuid, target, ProjectTransactionFault::None)
    }

    #[cfg(test)]
    pub(crate) fn relocate_project_source_with_fault(
        &mut self,
        source_uuid: AssetUuid,
        target: AssetUri,
        fault: ProjectTransactionFault,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let prepared = candidate.prepare_project_source_relocation(source_uuid, target)?;
        let updated = prepared.updated.clone();
        let outcome = prepared.commit_with_fault(fault)?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(updated)
    }

    #[cfg(test)]
    pub(crate) fn relocate_project_source_with_source_retirement_interruption(
        &mut self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.relocate_project_source_with_fault(
            source_uuid,
            target,
            ProjectTransactionFault::CrashAfterRetiredDelete(0),
        )
    }

    pub(crate) fn prepare_project_source_relocation(
        &mut self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<PreparedProjectSourceRelocation, AssetImportError> {
        let preflight = AssetMutationRelocationPreflight::evaluate(
            &self.asset_registry,
            source_uuid,
            target.clone(),
        );
        match preflight.disposition() {
            AssetMutationRelocationDisposition::Ready => {}
            AssetMutationRelocationDisposition::NoChange => {
                let source = preflight
                    .source()
                    .ok_or_else(|| {
                        AssetImportError::Parse(format!(
                            "project source relocation preflight omitted its unchanged source: {source_uuid}"
                        ))
                    })?
                    .locator()
                    .clone();
                return Ok(PreparedProjectSourceRelocation {
                    journal_directory: journal_directory(&self.paths),
                    writes: Vec::new(),
                    meta_paths: Vec::new(),
                    meta_preconditions: Vec::new(),
                    updated: Vec::new(),
                    source,
                    target,
                    target_path: PathBuf::new(),
                    locator_moves: Vec::new(),
                });
            }
            disposition => {
                return Err(relocation_preflight_error(
                    disposition,
                    source_uuid,
                    &target,
                ));
            }
        }
        let source = preflight
            .source()
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "project source relocation preflight omitted its ready source: {source_uuid}"
                ))
            })?
            .locator()
            .clone();
        let source_path = self.source_path_for_uri(&source)?;
        let target_path = self.primary_project_source_path_for_uri(&target)?;
        ensure_missing_target(&target_path)?;
        let source_meta_path = meta_path_for_source(&source_path);
        let target_meta_path = meta_path_for_source(&target_path);
        ensure_missing_target(&target_meta_path)?;

        let source_bytes = fs::read(&source_path)?;
        let expected_source_retirement_digest = blake3::hash(&source_bytes).to_hex().to_string();
        let source_meta = AssetMetaDocument::load(&source_meta_path)?;
        validate_relocatable_source(&source, &source_meta, &source_bytes)?;
        let (relocated_meta, locator_map) = relocate_meta_document(&source_meta, &source, &target)?;
        let (asset_registry, affected_uuids) = self
            .asset_registry
            .prepare_source_relocation_generation(&source, &relocated_meta)?;

        let mut registry = self.registry.begin_staging();
        for (previous, relocated) in &locator_map {
            registry
                .stage_rename_locator(previous, relocated.clone())
                .map_err(|error| {
                    AssetImportError::Parse(format!(
                        "project source relocation resource staging failed: {error}"
                    ))
                })?;
        }
        refresh_runtime_dependency_closure(&mut registry, &asset_registry, &affected_uuids)?;

        let root_id = AssetId::from_asset_uuid(source_uuid);
        let catalog_record = self
            .catalog_input_generation
            .record(root_id)
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "project source relocation is missing catalog input for {source}"
                ))
            })?;
        let direct_references = remap_references(catalog_record.direct_references(), &locator_map);
        let catalog_source = ProjectCatalogInputSource::new(
            target_path.clone(),
            target_meta_path.clone(),
            relocated_meta.clone(),
            catalog_record.source_mtime_unix_ms(),
            direct_references,
            catalog_record.reference_repairs().to_vec(),
        );
        let catalog_updated_records = affected_records(&registry, &affected_uuids);
        let catalog_input_generation = ProjectCatalogInputGeneration::publish_targeted(
            &self.catalog_input_generation,
            self.paths.root(),
            &self.manifest,
            &self.package_assets,
            catalog_updated_records,
            HashMap::from([(root_id, catalog_source)]),
            std::iter::empty(),
        );
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
        let updated = affected_records(&registry, &affected_uuids);

        self.registry = registry.finish();
        self.asset_registry = Arc::new(asset_registry);
        self.catalog_input_generation = catalog_input_generation;
        Ok(PreparedProjectSourceRelocation {
            journal_directory: journal_directory(&self.paths),
            writes: vec![
                PreparedFileWrite::new(target_path.clone(), source_bytes)
                    .retiring_with_expected_digest(source_path, expected_source_retirement_digest),
                PreparedFileWrite::new(target_meta_path.clone(), relocated_meta.to_pretty_bytes()?)
                    .retiring(source_meta_path.clone()),
                PreparedFileWrite::new(persisted.path, persisted.bytes),
            ],
            meta_paths: vec![source_meta_path.clone(), target_meta_path.clone()],
            meta_preconditions: vec![
                (source_meta_path, Some(source_meta)),
                (target_meta_path, None),
            ],
            updated,
            source,
            target,
            target_path,
            locator_moves: sorted_locator_moves(locator_map),
        })
    }
}

pub(super) fn validate_relocatable_source(
    source: &AssetUri,
    meta: &AssetMetaDocument,
    source_bytes: &[u8],
) -> Result<(), AssetImportError> {
    if meta.url != *source {
        return Err(AssetImportError::Parse(format!(
            "project source relocation metadata locator differs from the active registry: {} != {source}",
            meta.url
        )));
    }
    if meta.unit != AssetSourceUnit::Single || !meta.included_files.is_empty() {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "project source relocation requires a single-file source without included members: {source}"
        )));
    }
    if hash_bytes(source_bytes) != meta.source_digest {
        return Err(AssetImportError::Parse(format!(
            "project source relocation requires a current imported generation: {source}"
        )));
    }
    Ok(())
}

fn relocate_meta_document(
    source_meta: &AssetMetaDocument,
    source: &AssetUri,
    target: &AssetUri,
) -> Result<(AssetMetaDocument, HashMap<AssetUri, AssetUri>), AssetImportError> {
    let mut relocated = source_meta.clone();
    let mut locator_map = HashMap::with_capacity(relocated.entries.len() + 1);
    let relocated_root = relocate_owned_locator(source, target, &relocated.url)?;
    locator_map.insert(relocated.url.clone(), relocated_root.clone());
    relocated.url = relocated_root;
    for entry in &mut relocated.entries {
        let relocated_locator = relocate_owned_locator(source, target, &entry.url)?;
        locator_map.insert(entry.url.clone(), relocated_locator.clone());
        entry.url = relocated_locator;
    }
    remap_locator_paths(&mut relocated.dependencies, &locator_map);
    for entry in &mut relocated.entries {
        remap_locator_paths(&mut entry.dependencies, &locator_map);
    }
    Ok((relocated, locator_map))
}

fn relocate_owned_locator(
    source: &AssetUri,
    target: &AssetUri,
    locator: &AssetUri,
) -> Result<AssetUri, AssetImportError> {
    let base = AssetUri::new(locator.scheme(), locator.path().to_owned(), None)?;
    if base != *source {
        return Err(AssetImportError::Parse(format!(
            "project source relocation metadata entry does not belong to {source}: {locator}"
        )));
    }
    Ok(AssetUri::new(
        target.scheme(),
        target.path().to_owned(),
        locator.label().map(str::to_owned),
    )?)
}

fn remap_locator_paths(paths: &mut [AssetUri], locator_map: &HashMap<AssetUri, AssetUri>) {
    for path in paths {
        if let Some(relocated) = locator_map.get(path) {
            *path = relocated.clone();
        }
    }
}

fn sorted_locator_moves(locator_map: HashMap<AssetUri, AssetUri>) -> Vec<(AssetUri, AssetUri)> {
    let mut moves = locator_map.into_iter().collect::<Vec<_>>();
    moves.sort_by(|(left, _), (right, _)| left.cmp(right));
    moves
}

fn remap_references(
    references: &[AssetReference],
    locator_map: &HashMap<AssetUri, AssetUri>,
) -> Vec<AssetReference> {
    references
        .iter()
        .map(|reference| {
            AssetReference::new(
                reference.uuid,
                locator_map
                    .get(&reference.locator)
                    .cloned()
                    .unwrap_or_else(|| reference.locator.clone()),
            )
        })
        .collect()
}

fn affected_records(
    registry: &crate::core::resource::ResourceRegistryStaging,
    affected_uuids: &HashSet<AssetUuid>,
) -> Vec<ResourceRecord> {
    let mut records = affected_uuids
        .iter()
        .filter_map(|uuid| registry.get(AssetId::from_asset_uuid(*uuid)).cloned())
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.primary_locator().cmp(right.primary_locator()));
    records
}

pub(super) fn verify_meta_preconditions(
    preconditions: &[(PathBuf, Option<AssetMetaDocument>)],
) -> Result<(), AssetImportError> {
    for (path, expected) in preconditions {
        let current = match AssetMetaDocument::load(path) {
            Ok(document) => Some(document),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        if current.as_ref() != expected.as_ref() {
            return Err(AssetImportError::Parse(format!(
                "project source relocation metadata changed while prepared: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn ensure_missing_target(path: &Path) -> Result<(), AssetImportError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(AssetImportError::Parse(format!(
            "project source relocation target is already occupied: {}",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn relocation_preflight_error(
    disposition: AssetMutationRelocationDisposition,
    source_uuid: AssetUuid,
    target: &AssetUri,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "project source relocation preflight rejected {source_uuid} -> {target}: {disposition:?}"
    ))
}
