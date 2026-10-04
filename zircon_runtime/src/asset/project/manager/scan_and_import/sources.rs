use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::asset::project::ProjectGenerationObservation;
use crate::asset::project::{AssetMetaDocument, AssetMetaError, AssetSourceUnit};
use crate::asset::reference_resolver::persisted_source_path_for_locator;
use crate::asset::{AssetImportError, AssetUri};
use crate::core::resource::ResourceScheme;

use super::super::{
    collect_files::{
        collect_compound_files_with_limit, collect_files_excluding_subtrees, visit_matching_files,
    },
    meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms,
    ProjectManager,
};

// Compound source input follows the artifact store's raw payload ceiling. The member cap keeps
// zero-byte packages from creating an unbounded snapshot map while leaving normal authoring
// packages well below the established runtime payload limit.
const MAX_COMPOUND_SOURCE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_COMPOUND_SOURCE_FILES: usize =
    crate::asset::importer::AuxiliarySourceResolver::MAX_SNAPSHOT_FILES;

fn source_byte_limit() -> u64 {
    #[cfg(test)]
    return super::full_generation::source_snapshot_tests::byte_limit(MAX_COMPOUND_SOURCE_BYTES);
    #[cfg(not(test))]
    MAX_COMPOUND_SOURCE_BYTES
}

pub(super) struct AssetImportSource {
    pub(super) asset_root: PathBuf,
    pub(super) path: PathBuf,
    pub(super) uri: AssetUri,
    pub(super) meta_path: PathBuf,
    pub(super) unit: AssetSourceUnit,
    pub(super) included_files: Vec<AssetUri>,
    pub(super) included_paths: Vec<PathBuf>,
    pub(super) compound_root: Option<PathBuf>,
    pub(super) source_snapshot: Option<AssetImportSourceSnapshot>,
    snapshot_complete: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct AssetImportSourceSnapshot {
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) source_mtime_unix_ms: u64,
    pub(crate) source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
}

impl AssetImportSource {
    pub(crate) fn with_source_snapshot(mut self, snapshot: AssetImportSourceSnapshot) -> Self {
        self.source_snapshot = Some(snapshot);
        self.snapshot_complete = true;
        self
    }

    pub(super) fn source_file_snapshots(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        self.source_snapshot
            .as_ref()
            .map(|snapshot| snapshot.source_file_snapshots.clone())
            .unwrap_or_default()
    }

    pub(super) fn take_source_file_snapshots(&mut self) -> BTreeMap<PathBuf, Vec<u8>> {
        self.source_snapshot
            .as_mut()
            .map(|snapshot| std::mem::take(&mut snapshot.source_file_snapshots))
            .unwrap_or_default()
    }
}

impl ProjectManager {
    pub(super) fn prepare_targeted_import_source(
        &self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<AssetImportSource, AssetImportError> {
        let uri = AssetUri::new(uri.scheme(), uri.path().to_string(), None)?;
        let asset_root = match uri.package_id() {
            Some(package_id) => self
                .package_assets
                .root_for_package(package_id)
                .ok_or_else(|| {
                    targeted_full_scan(uri.clone(), "package source root is not registered")
                })?
                .to_path_buf(),
            None => self
                .package_assets
                .project_roots()
                .iter()
                .find(|root| indexed_path.starts_with(root))
                .cloned()
                .ok_or_else(|| {
                    targeted_full_scan(
                        uri.clone(),
                        "source is outside configured project asset roots",
                    )
                })?,
        };
        if !indexed_path.is_dir() {
            return Ok(AssetImportSource {
                asset_root,
                path: indexed_path.to_path_buf(),
                uri,
                meta_path: meta_path_for_source(indexed_path),
                unit: AssetSourceUnit::Single,
                included_files: Vec::new(),
                included_paths: Vec::new(),
                compound_root: None,
                source_snapshot: None,
                snapshot_complete: false,
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
        collect_compound_files_with_limit(
            indexed_path,
            &mut included_paths,
            MAX_COMPOUND_SOURCE_FILES,
        )?;
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
            asset_root,
            path: meta_path.clone(),
            uri,
            meta_path,
            unit: AssetSourceUnit::Compound,
            included_files,
            included_paths,
            compound_root: Some(indexed_path.to_path_buf()),
            source_snapshot: None,
            snapshot_complete: false,
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
            .collect::<HashSet<_>>();

        let mut files = Vec::new();
        collect_files_excluding_subtrees(root, &mut files, &compound_roots)?;
        for file in files {
            sources.push(AssetImportSource {
                asset_root: root.to_path_buf(),
                uri: self.source_uri_for_asset_root_path(root, package_id, &file)?,
                path: file.clone(),
                meta_path: meta_path_for_source(&file),
                unit: AssetSourceUnit::Single,
                included_files: Vec::new(),
                included_paths: Vec::new(),
                compound_root: None,
                source_snapshot: None,
                snapshot_complete: false,
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
        let mut sources = Vec::new();
        visit_matching_files(
            root,
            |path| {
                path.file_name()
                    .and_then(|file_name| file_name.to_str())
                    .is_some_and(|file_name| file_name.ends_with(".zmeta"))
            },
            |meta_path| {
                if let Some(source) =
                    self.compound_source_for_meta_path(root, package_id, observation, meta_path)?
                {
                    sources.push(source);
                }
                Ok(())
            },
        )?;
        Ok(sources)
    }

    fn compound_source_for_meta_path(
        &self,
        root: &Path,
        package_id: Option<&str>,
        observation: &mut ProjectGenerationObservation,
        meta_path: &Path,
    ) -> Result<Option<AssetImportSource>, AssetImportError> {
        let meta = match observation.load_metadata_document(meta_path) {
            Ok(meta) => meta,
            Err(error) if oversized_sidecar_has_compound_directory(meta_path, &error) => {
                return Err(AssetImportError::Io(error));
            }
            Err(_) => return Ok(None),
        };
        if meta.unit != AssetSourceUnit::Compound {
            return Ok(None);
        }
        let Some(compound_root) = compound_root_for_meta_path(meta_path) else {
            return Ok(None);
        };
        let uri = self.source_uri_for_asset_root_path(root, package_id, &compound_root)?;
        let Some(persisted_source) =
            persisted_source_path_for_locator(root, &uri).map_err(AssetImportError::Io)?
        else {
            return Ok(None);
        };
        if persisted_source != meta_path {
            return Ok(None);
        }
        let mut included_paths = Vec::new();
        collect_compound_files_with_limit(
            &compound_root,
            &mut included_paths,
            MAX_COMPOUND_SOURCE_FILES,
        )?;
        included_paths.sort();
        let included_files = included_paths
            .iter()
            .map(|path| self.source_uri_for_asset_root_path(root, package_id, path))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(AssetImportSource {
            asset_root: root.to_path_buf(),
            uri,
            path: meta_path.to_path_buf(),
            meta_path: meta_path.to_path_buf(),
            unit: AssetSourceUnit::Compound,
            included_files,
            included_paths,
            compound_root: Some(compound_root),
            source_snapshot: None,
            snapshot_complete: false,
        }))
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
    let mut paths_by_uri = HashMap::with_capacity(sources.len());
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
    let root_name = file_name
        .strip_suffix(".zmeta")
        .filter(|root_name| !root_name.is_empty())?;
    Some(meta_path.with_file_name(root_name))
}

fn oversized_sidecar_has_compound_directory(meta_path: &Path, error: &std::io::Error) -> bool {
    if !matches!(
        error
            .get_ref()
            .and_then(|source| source.downcast_ref::<AssetMetaError>()),
        Some(AssetMetaError::DocumentTooLarge { .. })
    ) {
        return false;
    }
    let Some(compound_root) = compound_root_for_meta_path(meta_path) else {
        return false;
    };
    match std::fs::symlink_metadata(compound_root) {
        Ok(metadata) => metadata.is_dir(),
        Err(source) => source.kind() != std::io::ErrorKind::NotFound,
    }
}

pub(super) fn source_bytes_for_import(
    source: &AssetImportSource,
) -> Result<Vec<u8>, AssetImportError> {
    if let Some(snapshot) = &source.source_snapshot {
        return Ok(snapshot.source_bytes.clone());
    }
    read_primary_source_snapshot(source).map(|(bytes, _)| bytes)
}

pub(super) fn materialize_compound_source_snapshot(
    source: &mut AssetImportSource,
) -> Result<(), AssetImportError> {
    if source.source_snapshot.is_some() {
        if source.snapshot_complete {
            return Ok(());
        }
        return Err(AssetImportError::Parse(format!(
            "source snapshot for {} is incomplete",
            source.uri
        )));
    }
    if source.compound_root.is_none() {
        let (source_bytes, source_mtime_unix_ms) = read_primary_source_snapshot(source)?;
        source.source_snapshot = Some(AssetImportSourceSnapshot {
            source_bytes,
            source_mtime_unix_ms,
            source_file_snapshots: BTreeMap::new(),
        });
        snapshot_declared_auxiliary_sources(source)?;
        source.snapshot_complete = true;
        return Ok(());
    }
    let compound_root = source
        .compound_root
        .as_ref()
        .expect("compound root was checked above");
    if source.included_paths.len() > MAX_COMPOUND_SOURCE_FILES {
        return Err(source_budget_error(
            &source.path,
            format!(
                "compound source contains {} files, exceeding the {}-file limit",
                source.included_paths.len(),
                MAX_COMPOUND_SOURCE_FILES
            ),
        ));
    }
    let (source_bytes, mut latest_mtime_unix_ms) = read_primary_source_snapshot(source)?;
    let mut total_bytes = source_bytes.len() as u64;
    // Retain the admitted primary even if a member fails, without publishing partial members.
    source.source_snapshot = Some(AssetImportSourceSnapshot {
        source_bytes,
        source_mtime_unix_ms: latest_mtime_unix_ms,
        source_file_snapshots: BTreeMap::new(),
    });
    let mut source_file_snapshots = BTreeMap::new();
    let resolver = crate::asset::importer::AuxiliarySourceResolver::for_asset_root(
        &source.asset_root,
        compound_root,
    )?;
    for included_path in &source.included_paths {
        let relative = included_path
            .strip_prefix(compound_root)
            .unwrap_or(included_path.as_path());
        let relative_bytes = relative.to_string_lossy();
        let overhead = 12_u64
            .checked_add(relative_bytes.len() as u64)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| source_budget_error(&source.path, "compound source size overflow"))?;
        let remaining = source_byte_limit()
            .checked_sub(total_bytes.saturating_add(overhead))
            .ok_or_else(|| {
                source_budget_error(
                    &source.path,
                    format!(
                        "compound source exceeds the {}-byte limit",
                        source_byte_limit()
                    ),
                )
            })?;
        let (_, included_bytes, included_mtime) =
            resolver.read_path_snapshot(relative, remaining)?;
        total_bytes = total_bytes
            .checked_add(overhead)
            .and_then(|value| value.checked_add(included_bytes.len() as u64))
            .ok_or_else(|| source_budget_error(&source.path, "compound source size overflow"))?;
        latest_mtime_unix_ms = latest_mtime_unix_ms.max(included_mtime);
        source_file_snapshots.insert(included_path.clone(), included_bytes);
    }
    let snapshot = source
        .source_snapshot
        .as_mut()
        .expect("primary source was captured before member admission");
    snapshot.source_mtime_unix_ms = latest_mtime_unix_ms;
    snapshot.source_file_snapshots = source_file_snapshots;
    snapshot_declared_auxiliary_sources(source)?;
    source.snapshot_complete = true;
    Ok(())
}

fn snapshot_declared_auxiliary_sources(
    source: &mut AssetImportSource,
) -> Result<(), AssetImportError> {
    let snapshot = source
        .source_snapshot
        .as_mut()
        .expect("the primary source is captured before auxiliary discovery");
    let retained = snapshot
        .source_file_snapshots
        .values()
        .try_fold(snapshot.source_bytes.len() as u64, |total, bytes| {
            total.checked_add(bytes.len() as u64)
        })
        .ok_or_else(|| source_budget_error(&source.path, "source snapshot size overflow"))?;
    let remaining = source_byte_limit().checked_sub(retained).ok_or_else(|| {
        source_budget_error(
            &source.path,
            format!(
                "source snapshot exceeds the {}-byte cumulative limit",
                source_byte_limit()
            ),
        )
    })?;
    let extension = source
        .path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let (additional, auxiliary_mtime_unix_ms) =
        if extension.eq_ignore_ascii_case("gltf") || extension.eq_ignore_ascii_case("glb") {
            crate::asset::importer::snapshot_external_gltf_sources(
                &source.asset_root,
                &source.path,
                &source.uri,
                &snapshot.source_bytes,
                &snapshot.source_file_snapshots,
                remaining,
            )?
        } else {
            #[cfg(any(feature = "graphics", feature = "target-server"))]
            if source.compound_root.is_some()
                && snapshot.source_file_snapshots.keys().any(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("zshader"))
                })
            {
                crate::asset::importer::snapshot_external_shader_sources(
                    &source.asset_root,
                    &source.path,
                    &source.uri,
                    &snapshot.source_file_snapshots,
                    remaining,
                )?
            } else {
                (BTreeMap::new(), 0)
            }
            #[cfg(not(any(feature = "graphics", feature = "target-server")))]
            (BTreeMap::new(), 0)
        };
    snapshot.source_mtime_unix_ms = snapshot.source_mtime_unix_ms.max(auxiliary_mtime_unix_ms);
    for (path, bytes) in additional {
        snapshot.source_file_snapshots.insert(path, bytes);
    }
    Ok(())
}

fn read_primary_source_snapshot(
    source: &AssetImportSource,
) -> Result<(Vec<u8>, u64), AssetImportError> {
    let parent = source
        .path
        .parent()
        .ok_or_else(|| AssetImportError::Parse("source has no parent".into()))?;
    let name = source
        .path
        .file_name()
        .ok_or_else(|| AssetImportError::Parse("source has no file name".into()))?;
    let resolver = crate::asset::importer::AuxiliarySourceResolver::for_asset_root(
        &source.asset_root,
        parent,
    )?;
    let (_, bytes, mtime) = resolver.read_path_snapshot(Path::new(name), source_byte_limit())?;
    Ok((bytes, mtime))
}

fn source_budget_error(path: &Path, reason: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(format!(
        "asset source {} exceeds import budget: {}",
        path.display(),
        reason.into()
    ))
}

pub(super) fn source_digest_for_import(
    source_bytes: &[u8],
    snapshots: &BTreeMap<PathBuf, Vec<u8>>,
    source_root: Option<&Path>,
) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_bytes.hash(&mut hasher);
    for (path, bytes) in snapshots {
        let relative = source_root
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path);
        relative.components().count().hash(&mut hasher);
        for component in relative.components() {
            match component.as_os_str().to_str() {
                Some(value) => {
                    0_u8.hash(&mut hasher);
                    value.hash(&mut hasher);
                }
                None => {
                    1_u8.hash(&mut hasher);
                    component.as_os_str().hash(&mut hasher);
                }
            }
        }
        bytes.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub(super) fn source_asset_root_for_digest(source: &AssetImportSource) -> PathBuf {
    source.asset_root.clone()
}

pub(super) fn take_source_bytes_for_import(
    source: &mut AssetImportSource,
) -> Result<Vec<u8>, AssetImportError> {
    if let Some(snapshot) = &mut source.source_snapshot {
        return Ok(std::mem::take(&mut snapshot.source_bytes));
    }
    source_bytes_for_import(source)
}

pub(super) fn source_mtime_unix_ms_for_import(
    source: &AssetImportSource,
) -> Result<u64, AssetImportError> {
    if let Some(snapshot) = &source.source_snapshot {
        return Ok(snapshot.source_mtime_unix_ms);
    }
    let mut mtime = source_mtime_unix_ms(&source.path)?;
    for included_path in &source.included_paths {
        mtime = mtime.max(source_mtime_unix_ms(included_path)?);
    }
    Ok(mtime)
}

#[cfg(test)]
#[path = "tests/sources.rs"]
mod tests;

#[cfg(test)]
#[path = "sources/tests/optimization_batch_je_runtime644_tests.rs"]
mod optimization_batch_je_runtime644_tests;

#[cfg(test)]
#[path = "sources/tests/snapshot_failure_tests.rs"]
mod snapshot_failure_tests;

#[cfg(test)]
#[path = "sources/tests/compound_discovery_tests.rs"]
mod compound_discovery_tests;
