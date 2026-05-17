use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry, AssetSourceUnit, PreviewState};
use crate::asset::{
    AssetId, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    ImportedAssetEntry,
};

use super::{
    asset_kind::asset_kind, collect_files::collect_files, hash_bytes::hash_bytes,
    load_or_create_meta::load_or_create_meta, meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms, ProjectManager,
};

impl ProjectManager {
    pub fn scan_and_import(&mut self) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let sources = self.collect_import_sources()?;

        let mut registry = ResourceRegistry::default();
        let mut asset_ids_by_uuid = HashMap::new();
        let mut asset_uuids_by_id = HashMap::new();
        let mut dependencies_by_id = HashMap::new();
        let mut imported = Vec::with_capacity(sources.len());
        self.asset_urls_by_uuid.clear();

        for source in sources {
            let file = source.path.clone();
            let uri = source.uri.clone();
            let source_bytes = source_bytes_for_import(&source)?;
            let source_hash = hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms_for_import(&source)?;
            let descriptor = self.importer.descriptor_for_source(&file).ok();
            let fallback_kind = descriptor
                .as_ref()
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let meta_path = source.meta_path.clone();
            let meta_exists = meta_path.exists();
            let mut meta = load_or_create_meta(&meta_path, &uri, fallback_kind)?;
            let previous_meta = meta.clone();
            meta.unit = source.unit;
            meta.included_files = source.included_files.clone();
            let import_settings = meta.import_settings.clone();
            let config_hash = config_hash_for_settings(&import_settings);
            let root_asset_id = AssetId::from_asset_uuid(meta.uuid);

            if let Some(metadata) = self.restore_imported_artifact(
                &source,
                &mut meta,
                meta_exists,
                &previous_meta,
                source_hash.clone(),
                source_mtime_unix_ms,
                config_hash.clone(),
                descriptor.as_ref(),
                fallback_kind,
            )? {
                for record in metadata {
                    let asset_id = record.id();
                    dependencies_by_id.insert(
                        asset_id,
                        dependencies_for_entry(&meta, record.primary_locator()),
                    );
                    registry.upsert(record.clone());
                    register_record_identity(
                        &mut asset_ids_by_uuid,
                        &mut asset_uuids_by_id,
                        &mut self.asset_urls_by_uuid,
                        &meta,
                        &record,
                    );
                    imported.push(record);
                }
                continue;
            }

            let import_result =
                self.importer
                    .import_bytes(&file, &uri, source_bytes, import_settings);
            let metadata = match import_result {
                Ok(outcome) => match validate_import_entries(&uri, &outcome) {
                    Ok(()) => self.finish_successful_import(
                        &source,
                        &mut meta,
                        meta_exists,
                        &previous_meta,
                        source_hash.clone(),
                        source_mtime_unix_ms,
                        config_hash,
                        descriptor.as_ref(),
                        outcome,
                    )?,
                    Err(error) => self.finish_failed_import(
                        &source,
                        &mut meta,
                        meta_exists,
                        &previous_meta,
                        source_hash.clone(),
                        source_mtime_unix_ms,
                        config_hash,
                        descriptor.as_ref(),
                        fallback_kind,
                        root_asset_id,
                        error,
                    )?,
                },
                Err(error) => self.finish_failed_import(
                    &source,
                    &mut meta,
                    meta_exists,
                    &previous_meta,
                    source_hash.clone(),
                    source_mtime_unix_ms,
                    config_hash,
                    descriptor.as_ref(),
                    fallback_kind,
                    root_asset_id,
                    error,
                )?,
            };
            for record in metadata {
                let asset_id = record.id();
                dependencies_by_id.insert(
                    asset_id,
                    dependencies_for_entry(&meta, record.primary_locator()),
                );
                registry.upsert(record.clone());
                register_record_identity(
                    &mut asset_ids_by_uuid,
                    &mut asset_uuids_by_id,
                    &mut self.asset_urls_by_uuid,
                    &meta,
                    &record,
                );
                imported.push(record);
            }
        }

        resolve_imported_dependencies(&mut registry, &mut imported, &dependencies_by_id);

        self.registry = registry;
        self.asset_ids_by_uuid = asset_ids_by_uuid;
        self.asset_uuids_by_id = asset_uuids_by_id;
        Ok(imported)
    }

    fn collect_import_sources(&self) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut sources = Vec::new();
        self.collect_import_sources_for_root(self.paths.assets_root(), None, &mut sources)?;

        for (package_id, root) in self.package_assets.iter() {
            self.collect_import_sources_for_root(root, Some(package_id), &mut sources)?;
        }

        sources.sort_by(|left, right| left.uri.cmp(&right.uri));
        Ok(sources)
    }

    fn collect_import_sources_for_root(
        &self,
        root: &Path,
        package_id: Option<&str>,
        sources: &mut Vec<AssetImportSource>,
    ) -> Result<(), AssetImportError> {
        let mut compound_sources = self.collect_compound_sources_for_root(root, package_id)?;
        let compound_roots = compound_sources
            .iter()
            .filter_map(|source| source.compound_root.clone())
            .collect::<Vec<_>>();

        let mut files = Vec::new();
        collect_files(root, &mut files)?;
        for file in files {
            if compound_roots
                .iter()
                .any(|compound_root| file.starts_with(compound_root))
            {
                continue;
            }
            sources.push(AssetImportSource {
                uri: self.source_uri_for_asset_root_path(root, package_id, &file)?,
                path: file.clone(),
                meta_path: meta_path_for_source(&file),
                unit: AssetSourceUnit::Single,
                included_files: Vec::new(),
                included_paths: Vec::new(),
                compound_root: None,
            });
        }

        sources.append(&mut compound_sources);
        Ok(())
    }

    fn collect_compound_sources_for_root(
        &self,
        root: &Path,
        package_id: Option<&str>,
    ) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut meta_files = Vec::new();
        collect_zmeta_files(root, &mut meta_files)?;
        let mut sources = Vec::new();

        for meta_path in meta_files {
            let Ok(meta) = AssetMetaDocument::load(&meta_path) else {
                continue;
            };
            if meta.unit != AssetSourceUnit::Compound {
                continue;
            }
            let Some(compound_root) = compound_root_for_meta_path(&meta_path) else {
                continue;
            };
            let mut included_paths = Vec::new();
            collect_files(&compound_root, &mut included_paths)?;
            included_paths.sort();
            let included_files = included_paths
                .iter()
                .map(|path| self.source_uri_for_asset_root_path(root, package_id, path))
                .collect::<Result<Vec<_>, _>>()?;
            sources.push(AssetImportSource {
                uri: self.source_uri_for_asset_root_path(root, package_id, &compound_root)?,
                path: meta_path.clone(),
                meta_path,
                unit: AssetSourceUnit::Compound,
                included_files,
                included_paths,
                compound_root: Some(compound_root),
            });
        }

        Ok(sources)
    }

    fn source_uri_for_asset_root_path(
        &self,
        root: &Path,
        package_id: Option<&str>,
        path: &Path,
    ) -> Result<crate::asset::AssetUri, AssetImportError> {
        if let Some(package_id) = package_id {
            self.source_uri_for_package_path(package_id, root, path)
        } else {
            self.source_uri_for_path(path)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn restore_imported_artifact(
        &self,
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        fallback_kind: AssetKind,
    ) -> Result<Option<Vec<ResourceRecord>>, AssetImportError> {
        let uri = &source.uri;
        if meta.preview_state != PreviewState::Ready
            || meta.source_hash != source_hash
            || meta.config_hash != config_hash
            || !importer_contract_matches(meta, descriptor)
        {
            return Ok(None);
        }

        if meta.entries.is_empty() {
            let Some(artifact_uri) = meta.artifact_locator.clone() else {
                return Ok(None);
            };
            meta.entries = vec![AssetMetaEntry {
                uuid: meta.uuid,
                url: uri.clone(),
                asset_kind: meta.asset_kind,
                artifact_locator: Some(artifact_uri),
                dependencies: meta.dependencies.clone(),
            }];
        }
        remap_meta_entry_urls_to_source(meta, uri);

        for entry in &meta.entries {
            let Some(artifact_uri) = &entry.artifact_locator else {
                return Ok(None);
            };
            if self.artifact_store.read(&self.paths, artifact_uri).is_err() {
                return Ok(None);
            }
        }

        meta.url = uri.clone();
        if meta.asset_kind == AssetKind::Data && descriptor.is_some() {
            meta.asset_kind = fallback_kind;
        }
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        if !meta_exists || meta != previous_meta {
            meta.save(&source.meta_path)?;
        }

        Ok(Some(
            meta.entries
                .iter()
                .map(|entry| {
                    let entry_asset_id = asset_id_for_meta_entry(entry);
                    let mut record =
                        ResourceRecord::new(entry_asset_id, entry.asset_kind, entry.url.clone())
                            .with_source_hash(source_hash.clone())
                            .with_importer_id(meta.importer_id.clone())
                            .with_importer_version(meta.importer_version)
                            .with_config_hash(config_hash.clone())
                            .with_state(ResourceState::Ready);
                    if let Some(artifact_uri) = entry.artifact_locator.clone() {
                        record = record.with_artifact_locator(artifact_uri);
                    }
                    record
                })
                .collect(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_successful_import(
        &self,
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        outcome: AssetImportOutcome,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let uri = &source.uri;
        let root_entry = outcome.root_entry().ok_or_else(|| {
            AssetImportError::Parse(format!("importer did not return a root entry for {uri}"))
        })?;
        let kind = asset_kind(&root_entry.asset);
        apply_importer_metadata(meta, descriptor);
        if let Some(migration) = &root_entry.migration_report {
            meta.source_schema_version = migration.source_schema_version;
            meta.target_schema_version = Some(migration.target_schema_version);
            meta.migration_summary = migration.summary.clone();
        } else {
            clear_schema_migration_metadata(meta);
        }
        meta.url = uri.clone();
        meta.asset_kind = kind;
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        meta.artifact_locator = None;
        meta.dependencies = root_entry.dependencies.clone();
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Ready;

        let mut entries = Vec::with_capacity(outcome.entries.len());
        let mut records = Vec::with_capacity(outcome.entries.len());
        let existing_entry_uuids = existing_entry_uuids_for_source(previous_meta, uri);
        for entry in outcome.entries {
            let entry_kind = asset_kind(&entry.asset);
            let entry_uuid = entry_uuid_for_import_entry(meta.uuid, &existing_entry_uuids, &entry);
            let entry_asset_id = AssetId::from_asset_uuid(entry_uuid);
            let artifact_record =
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator.clone());
            let artifact_uri =
                self.artifact_store
                    .write(&self.paths, &artifact_record, &entry.asset)?;
            if entry.locator.label().is_none() {
                meta.artifact_locator = Some(artifact_uri.clone());
            }
            entries.push(AssetMetaEntry {
                uuid: entry_uuid,
                url: entry.locator.clone(),
                asset_kind: entry_kind,
                artifact_locator: Some(artifact_uri.clone()),
                dependencies: entry.dependencies.clone(),
            });
            records.push(
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator)
                    .with_source_hash(source_hash.clone())
                    .with_importer_id(meta.importer_id.clone())
                    .with_importer_version(meta.importer_version)
                    .with_config_hash(config_hash.clone())
                    .with_artifact_locator(artifact_uri)
                    .with_state(ResourceState::Ready)
                    .with_diagnostics(entry.diagnostics),
            );
        }
        meta.entries = entries;
        if !meta_exists || meta != previous_meta {
            meta.save(&source.meta_path)?;
        }

        Ok(records)
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_failed_import(
        &self,
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        kind: AssetKind,
        asset_id: AssetId,
        error: AssetImportError,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let uri = &source.uri;
        apply_importer_metadata(meta, descriptor);
        clear_schema_migration_metadata(meta);
        meta.url = uri.clone();
        meta.asset_kind = kind;
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        meta.artifact_locator = None;
        meta.dependencies.clear();
        meta.entries = failed_entries_for_source(previous_meta, meta.uuid, uri, kind);
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Error;
        if !meta_exists || meta != previous_meta {
            meta.save(&source.meta_path)?;
        }

        Ok(vec![ResourceRecord::new(asset_id, kind, uri.clone())
            .with_source_hash(source_hash)
            .with_importer_id(meta.importer_id.clone())
            .with_importer_version(meta.importer_version)
            .with_config_hash(config_hash)
            .with_state(ResourceState::Error)
            .with_diagnostics(vec![ResourceDiagnostic::error(
                error.to_string(),
            )])])
    }
}

struct AssetImportSource {
    path: PathBuf,
    uri: crate::asset::AssetUri,
    meta_path: PathBuf,
    unit: AssetSourceUnit,
    included_files: Vec<crate::asset::AssetUri>,
    included_paths: Vec<PathBuf>,
    compound_root: Option<PathBuf>,
}

fn collect_zmeta_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_zmeta_files(&path, files)?;
        } else if path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with(".zmeta"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn compound_root_for_meta_path(meta_path: &Path) -> Option<PathBuf> {
    let file_name = meta_path.file_name()?.to_str()?;
    let root_name = file_name.strip_suffix(".zmeta")?;
    Some(meta_path.with_file_name(root_name))
}

fn source_bytes_for_import(source: &AssetImportSource) -> Result<Vec<u8>, AssetImportError> {
    let mut bytes = fs::read(&source.path)?;
    let Some(compound_root) = &source.compound_root else {
        return Ok(bytes);
    };

    for included_path in &source.included_paths {
        let relative = included_path
            .strip_prefix(compound_root)
            .unwrap_or(included_path.as_path());
        bytes.extend_from_slice(b"\n# included ");
        bytes.extend_from_slice(relative.to_string_lossy().as_bytes());
        bytes.extend_from_slice(b"\n");
        bytes.extend_from_slice(&fs::read(included_path)?);
    }
    Ok(bytes)
}

fn source_mtime_unix_ms_for_import(source: &AssetImportSource) -> Result<u64, AssetImportError> {
    let mut mtime = source_mtime_unix_ms(&source.path)?;
    for included_path in &source.included_paths {
        mtime = mtime.max(source_mtime_unix_ms(included_path)?);
    }
    Ok(mtime)
}

fn clear_schema_migration_metadata(meta: &mut crate::asset::project::AssetMetaDocument) {
    meta.source_schema_version = None;
    meta.target_schema_version = None;
    meta.migration_summary.clear();
}

fn apply_importer_metadata(
    meta: &mut crate::asset::project::AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) {
    if let Some(descriptor) = descriptor {
        meta.importer_id = descriptor.id.clone();
        meta.importer_version = descriptor.importer_version;
    } else {
        meta.importer_id.clear();
        meta.importer_version = 0;
    }
}

#[derive(Default)]
struct ResolvedDependencies {
    dependency_ids: Vec<AssetId>,
    diagnostics: Vec<ResourceDiagnostic>,
}

fn resolve_dependencies(
    dependencies: &[crate::asset::AssetUri],
    registry: &ResourceRegistry,
) -> ResolvedDependencies {
    let mut resolved = ResolvedDependencies::default();
    for dependency in dependencies {
        if let Some(record) = registry.get_by_locator(dependency) {
            if !resolved.dependency_ids.contains(&record.id()) {
                resolved.dependency_ids.push(record.id());
            }
        } else {
            resolved.diagnostics.push(ResourceDiagnostic::error(format!(
                "unresolved asset dependency {dependency}"
            )));
        }
    }
    resolved
}

fn resolve_imported_dependencies(
    registry: &mut ResourceRegistry,
    imported: &mut [ResourceRecord],
    dependencies_by_id: &HashMap<AssetId, Vec<crate::asset::AssetUri>>,
) {
    let resolved_by_id = dependencies_by_id
        .iter()
        .map(|(id, dependencies)| (*id, resolve_dependencies(dependencies, registry)))
        .collect::<HashMap<_, _>>();

    for record in imported.iter_mut() {
        apply_resolved_dependencies(record, &resolved_by_id);
        registry.upsert(record.clone());
    }
}

fn apply_resolved_dependencies(
    record: &mut ResourceRecord,
    resolved_by_id: &HashMap<AssetId, ResolvedDependencies>,
) {
    let Some(resolved) = resolved_by_id.get(&record.id()) else {
        return;
    };
    record.dependency_ids = resolved.dependency_ids.clone();
    record
        .diagnostics
        .extend(resolved.diagnostics.iter().cloned());
}

fn dependencies_for_entry(
    meta: &crate::asset::project::AssetMetaDocument,
    locator: &crate::asset::AssetUri,
) -> Vec<crate::asset::AssetUri> {
    meta.entries
        .iter()
        .find(|entry| &entry.url == locator)
        .map(|entry| entry.dependencies.clone())
        .unwrap_or_else(|| meta.dependencies.clone())
}

fn asset_id_for_meta_entry(entry: &AssetMetaEntry) -> AssetId {
    AssetId::from_asset_uuid(entry.uuid)
}

fn validate_import_entries(
    source_uri: &crate::asset::AssetUri,
    outcome: &AssetImportOutcome,
) -> Result<(), AssetImportError> {
    if outcome.entries.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "importer did not return any asset entries for {source_uri}"
        )));
    }

    let mut labels = HashSet::new();
    let mut root_count = 0;
    for entry in &outcome.entries {
        if entry.locator.scheme() != source_uri.scheme()
            || entry.locator.path() != source_uri.path()
        {
            return Err(AssetImportError::Parse(format!(
                "imported asset entry locator {} does not belong to source {source_uri}",
                entry.locator
            )));
        }
        match entry.locator.label() {
            Some(label) => {
                if !labels.insert(label.to_string()) {
                    return Err(AssetImportError::DuplicateAssetLabel {
                        source_uri: source_uri.clone(),
                        label: label.to_string(),
                    });
                }
            }
            None => root_count += 1,
        }
    }
    if root_count != 1 {
        return Err(AssetImportError::Parse(format!(
            "importer returned {root_count} root entries for {source_uri}; expected exactly one"
        )));
    }
    Ok(())
}

fn existing_entry_uuids_for_source(
    meta: &crate::asset::project::AssetMetaDocument,
    source_uri: &crate::asset::AssetUri,
) -> HashMap<crate::asset::AssetUri, crate::asset::AssetUuid> {
    meta.entries
        .iter()
        .map(|entry| (entry_url_for_source(&entry.url, source_uri), entry.uuid))
        .collect()
}

fn failed_entries_for_source(
    previous_meta: &crate::asset::project::AssetMetaDocument,
    root_uuid: crate::asset::AssetUuid,
    source_uri: &crate::asset::AssetUri,
    root_kind: AssetKind,
) -> Vec<AssetMetaEntry> {
    if previous_meta.entries.is_empty() {
        return Vec::new();
    }

    previous_meta
        .entries
        .iter()
        .map(|entry| AssetMetaEntry {
            uuid: if entry.url.label().is_none() {
                root_uuid
            } else {
                entry.uuid
            },
            url: entry_url_for_source(&entry.url, source_uri),
            asset_kind: if entry.url.label().is_none() {
                root_kind
            } else {
                entry.asset_kind
            },
            artifact_locator: None,
            dependencies: entry.dependencies.clone(),
        })
        .collect()
}

fn remap_meta_entry_urls_to_source(
    meta: &mut crate::asset::project::AssetMetaDocument,
    source_uri: &crate::asset::AssetUri,
) {
    for entry in &mut meta.entries {
        entry.url = entry_url_for_source(&entry.url, source_uri);
    }
}

fn entry_url_for_source(
    entry_url: &crate::asset::AssetUri,
    source_uri: &crate::asset::AssetUri,
) -> crate::asset::AssetUri {
    if entry_url.label().is_none() {
        source_uri.clone()
    } else {
        crate::asset::AssetUri::new(
            source_uri.scheme(),
            source_uri.path().to_string(),
            entry_url.label().map(ToOwned::to_owned),
        )
        .expect("source URI with existing entry label should be a valid asset URI")
    }
}

fn entry_uuid_for_import_entry(
    root_uuid: crate::asset::AssetUuid,
    existing_entry_uuids: &HashMap<crate::asset::AssetUri, crate::asset::AssetUuid>,
    entry: &ImportedAssetEntry,
) -> crate::asset::AssetUuid {
    if entry.locator.label().is_none() {
        root_uuid
    } else {
        existing_entry_uuids
            .get(&entry.locator)
            .copied()
            .unwrap_or_else(crate::asset::AssetUuid::new)
    }
}

fn register_record_identity(
    asset_ids_by_uuid: &mut HashMap<crate::asset::AssetUuid, AssetId>,
    asset_uuids_by_id: &mut HashMap<AssetId, crate::asset::AssetUuid>,
    asset_urls_by_uuid: &mut HashMap<crate::asset::AssetUuid, crate::asset::AssetUri>,
    meta: &crate::asset::project::AssetMetaDocument,
    record: &ResourceRecord,
) {
    let uuid = meta
        .entries
        .iter()
        .find(|entry| entry.url == *record.primary_locator())
        .map(|entry| entry.uuid)
        .unwrap_or(meta.uuid);
    asset_ids_by_uuid.insert(uuid, record.id());
    asset_uuids_by_id.insert(record.id(), uuid);
    asset_urls_by_uuid.insert(uuid, record.primary_locator().clone());
}

fn importer_contract_matches(
    meta: &crate::asset::project::AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) -> bool {
    descriptor
        .map(|descriptor| {
            meta.importer_id == descriptor.id
                && meta.importer_version == descriptor.importer_version
        })
        .unwrap_or_else(|| !meta.importer_id.is_empty())
}

fn config_hash_for_settings(settings: &toml::Table) -> String {
    toml::to_string(settings)
        .map(|document| hash_bytes(document.as_bytes()))
        .unwrap_or_default()
}
