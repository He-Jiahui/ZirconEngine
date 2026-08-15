use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::project::ProjectGenerationObservation;
use crate::asset::project::{AssetMetaDocument, AssetSourceUnit};
use crate::asset::reference_resolver::persisted_source_path_for_locator;
use crate::asset::{AssetImportError, AssetUri};
use crate::core::resource::ResourceScheme;

use super::super::{
    collect_files::{collect_files, collect_matching_files},
    meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms,
    ProjectManager,
};

pub(super) struct AssetImportSource {
    pub(super) path: PathBuf,
    pub(super) uri: AssetUri,
    pub(super) meta_path: PathBuf,
    pub(super) unit: AssetSourceUnit,
    pub(super) included_files: Vec<AssetUri>,
    pub(super) included_paths: Vec<PathBuf>,
    pub(super) compound_root: Option<PathBuf>,
}

impl ProjectManager {
    pub(super) fn prepare_targeted_import_source(
        &self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<AssetImportSource, AssetImportError> {
        let uri = AssetUri::new(uri.scheme(), uri.path().to_string(), None)?;
        if !indexed_path.is_dir() {
            return Ok(AssetImportSource {
                path: indexed_path.to_path_buf(),
                uri,
                meta_path: meta_path_for_source(indexed_path),
                unit: AssetSourceUnit::Single,
                included_files: Vec::new(),
                included_paths: Vec::new(),
                compound_root: None,
            });
        }

        let meta_path = meta_path_for_source(indexed_path);
        let meta = AssetMetaDocument::load(&meta_path)?;
        if meta.unit != AssetSourceUnit::Compound || meta.url != uri {
            return Err(targeted_full_scan(
                uri,
                "indexed directory does not match its compound source descriptor",
            ));
        }
        let mut included_paths = Vec::new();
        collect_files(indexed_path, &mut included_paths)?;
        included_paths.sort();
        let included_files =
            self.included_uris_for_targeted_source(&uri, indexed_path, &included_paths)?;
        if included_files != meta.included_files {
            return Err(targeted_full_scan(
                uri,
                "compound source membership changed since the active generation",
            ));
        }
        Ok(AssetImportSource {
            path: meta_path.clone(),
            uri,
            meta_path,
            unit: AssetSourceUnit::Compound,
            included_files,
            included_paths,
            compound_root: Some(indexed_path.to_path_buf()),
        })
    }

    fn included_uris_for_targeted_source(
        &self,
        uri: &AssetUri,
        compound_root: &Path,
        included_paths: &[PathBuf],
    ) -> Result<Vec<AssetUri>, AssetImportError> {
        match uri.scheme() {
            ResourceScheme::Res => {
                self.resolve_project_source_path(compound_root)
                    .map_err(|error| {
                        targeted_full_scan(
                            uri.clone(),
                            format!(
                                "compound members do not belong to one unambiguous project root: {error}"
                            ),
                        )
                    })?;
                included_paths
                    .iter()
                    .map(|path| self.project_uri_for_source_path(path))
                    .collect()
            }
            ResourceScheme::Package => {
                let package_id = uri.package_id().ok_or_else(|| {
                    targeted_full_scan(uri.clone(), "package source is missing a package id")
                })?;
                let root = self
                    .package_assets
                    .root_for_package(package_id)
                    .ok_or_else(|| {
                        targeted_full_scan(uri.clone(), "package source root is not registered")
                    })?;
                included_paths
                    .iter()
                    .map(|path| self.source_uri_for_package_path(package_id, root, path))
                    .collect()
            }
            _ => Err(targeted_full_scan(
                uri.clone(),
                "only project and package sources support targeted import",
            )),
        }
    }

    pub(super) fn collect_import_sources(
        &self,
        observation: &mut ProjectGenerationObservation,
    ) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut sources = Vec::new();
        let project_roots = self.package_assets.project_roots().to_vec();
        for root in &project_roots {
            self.collect_import_sources_for_root(root, None, &mut sources, observation)?;
        }

        for (package_id, root) in self.package_assets.iter() {
            self.collect_import_sources_for_root(
                root,
                Some(package_id),
                &mut sources,
                observation,
            )?;
        }

        reject_duplicate_project_uris(&sources)?;
        sources.sort_by(|left, right| left.uri.cmp(&right.uri));
        Ok(sources)
    }

    fn collect_import_sources_for_root(
        &self,
        root: &Path,
        package_id: Option<&str>,
        sources: &mut Vec<AssetImportSource>,
        observation: &mut ProjectGenerationObservation,
    ) -> Result<(), AssetImportError> {
        let mut compound_sources =
            self.collect_compound_sources_for_root(root, package_id, observation)?;
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
        observation: &mut ProjectGenerationObservation,
    ) -> Result<Vec<AssetImportSource>, AssetImportError> {
        let mut meta_files = Vec::new();
        collect_matching_files(root, &mut meta_files, |path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| file_name.ends_with(".zmeta"))
        })?;
        let mut sources = Vec::new();

        for meta_path in meta_files {
            let Ok(meta) = observation.load_metadata_document(&meta_path) else {
                continue;
            };
            if meta.unit != AssetSourceUnit::Compound {
                continue;
            }
            let Some(compound_root) = compound_root_for_meta_path(&meta_path) else {
                continue;
            };
            let uri = self.source_uri_for_asset_root_path(root, package_id, &compound_root)?;
            let Some(persisted_source) =
                persisted_source_path_for_locator(root, &uri).map_err(AssetImportError::Io)?
            else {
                continue;
            };
            if persisted_source != meta_path {
                continue;
            }
            let mut included_paths = Vec::new();
            collect_files(&compound_root, &mut included_paths)?;
            included_paths.sort();
            let included_files = included_paths
                .iter()
                .map(|path| self.source_uri_for_asset_root_path(root, package_id, path))
                .collect::<Result<Vec<_>, _>>()?;
            sources.push(AssetImportSource {
                uri,
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
    ) -> Result<AssetUri, AssetImportError> {
        if let Some(package_id) = package_id {
            self.source_uri_for_package_path(package_id, root, path)
        } else {
            self.project_uri_for_source_path(path)
        }
    }
}

fn targeted_full_scan(uri: AssetUri, reason: impl Into<String>) -> AssetImportError {
    AssetImportError::TargetedImportRequiresFullScan {
        uri,
        reason: reason.into(),
    }
}

fn reject_duplicate_project_uris(sources: &[AssetImportSource]) -> Result<(), AssetImportError> {
    let mut paths_by_uri = BTreeMap::new();
    for source in sources {
        if source.uri.package_id().is_some() {
            continue;
        }
        if let Some(previous) = paths_by_uri.insert(source.uri.clone(), source.path.clone()) {
            return Err(AssetImportError::DuplicateProjectAssetUri {
                uri: source.uri.clone(),
                first: previous,
                second: source.path.clone(),
            });
        }
    }
    Ok(())
}

fn compound_root_for_meta_path(meta_path: &Path) -> Option<PathBuf> {
    let file_name = meta_path.file_name()?.to_str()?;
    let root_name = file_name.strip_suffix(".zmeta")?;
    Some(meta_path.with_file_name(root_name))
}

pub(super) fn source_bytes_for_import(
    source: &AssetImportSource,
) -> Result<Vec<u8>, AssetImportError> {
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

pub(super) fn source_mtime_unix_ms_for_import(
    source: &AssetImportSource,
) -> Result<u64, AssetImportError> {
    let mut mtime = source_mtime_unix_ms(&source.path)?;
    for included_path in &source.included_paths {
        mtime = mtime.max(source_mtime_unix_ms(included_path)?);
    }
    Ok(mtime)
}
