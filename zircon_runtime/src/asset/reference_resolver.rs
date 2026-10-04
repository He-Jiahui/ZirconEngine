use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::{AssetRef, RelPath};

use crate::asset::project::{AssetMetaDocument, AssetSourceUnit, ProjectPaths};
use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::safe_project_path::is_safe_regular_file;
use crate::asset::{AssetReference, AssetUri, ReferenceResolutionError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRepairKind {
    PathHint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceRepair {
    pub stale: AssetRef,
    pub resolved: AssetRef,
    pub kind: ReferenceRepairKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedProjectReference {
    pub(crate) reference: AssetReference,
    pub(crate) repair: Option<ReferenceRepair>,
}

/// Lookup boundary for the physical source projection owned by one project generation.
///
/// The shared reference-resolution algorithm consumes this contract so filesystem-backed runtime
/// callers and the migration generation keep one GUID/path/repair truth.
pub(crate) trait ProjectSourceLookup {
    fn project_hint_for_locator(
        &self,
        locator: &AssetUri,
    ) -> Result<RelPath, ReferenceResolutionError>;

    fn locator_for_project_hint(
        &self,
        hint: &RelPath,
    ) -> Result<Option<AssetUri>, ReferenceResolutionError>;
}

/// Maps a registry locator to its one persisted source file under an asset root.
///
/// A single-file asset persists its source file directly. A compound asset persists the
/// `.zmeta` file whose validated `unit` and logical URL describe the directory-root locator.
/// Directories themselves are deliberately never accepted as persisted sources.
pub(crate) fn persisted_source_path_for_locator(
    root: &Path,
    locator: &AssetUri,
) -> Result<Option<PathBuf>, std::io::Error> {
    let regular_source = root.join(locator.path());
    if is_safe_regular_file(root, &regular_source)? {
        return Ok(Some(regular_source));
    }

    let compound_meta = compound_meta_path(&regular_source)?;
    if !is_safe_regular_file(root, &compound_meta)? {
        return Ok(None);
    }
    let meta = AssetMetaDocument::load(&compound_meta)?;
    if meta.unit == AssetSourceUnit::Compound
        && meta.url.label().is_none()
        && meta.url.scheme() == locator.scheme()
        && meta.url.path() == locator.path()
    {
        Ok(Some(compound_meta))
    } else {
        Ok(None)
    }
}

/// Resolves the logical asset locator carried by a compound persisted `.zmeta` source.
pub(crate) fn logical_locator_for_persisted_source(
    root: &Path,
    source: &Path,
) -> Result<Option<AssetUri>, std::io::Error> {
    if !is_safe_regular_file(root, source)? {
        return Ok(None);
    }
    let Some(file_name) = source.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    if !file_name.ends_with(".zmeta") {
        return Ok(None);
    }
    let meta = AssetMetaDocument::load(source)?;
    if meta.unit != AssetSourceUnit::Compound || meta.url.label().is_some() {
        return Ok(None);
    }
    let expected = compound_meta_path(&root.join(meta.url.path()))?;
    if ProjectPaths::resolve_existing_path(source)?
        != ProjectPaths::resolve_existing_path(&expected)?
    {
        return Ok(None);
    }
    Ok(Some(meta.url))
}

fn compound_meta_path(logical_root: &Path) -> Result<PathBuf, std::io::Error> {
    let file_name = logical_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "compound logical root {} has no UTF-8 file name",
                    logical_root.display()
                ),
            )
        })?;
    Ok(logical_root.with_file_name(format!("{file_name}.zmeta")))
}

struct FilesystemProjectSourceLookup<'a> {
    roots: &'a [(RelPath, PathBuf)],
}

impl ProjectSourceLookup for FilesystemProjectSourceLookup<'_> {
    fn project_hint_for_locator(
        &self,
        locator: &AssetUri,
    ) -> Result<RelPath, ReferenceResolutionError> {
        filesystem_project_hint_for_locator(self.roots, locator)
    }

    fn locator_for_project_hint(
        &self,
        hint: &RelPath,
    ) -> Result<Option<AssetUri>, ReferenceResolutionError> {
        filesystem_locator_for_project_hint(self.roots, hint)
    }
}

pub(crate) fn resolve_project_reference(
    registry: &AssetRegistryIndex,
    roots: &[(RelPath, PathBuf)],
    reference: &AssetRef,
) -> Result<ResolvedProjectReference, ReferenceResolutionError> {
    resolve_project_reference_from_lookup(
        registry,
        &FilesystemProjectSourceLookup { roots },
        reference,
    )
}

pub(crate) fn resolve_project_reference_from_lookup(
    registry: &AssetRegistryIndex,
    sources: &impl ProjectSourceLookup,
    reference: &AssetRef,
) -> Result<ResolvedProjectReference, ReferenceResolutionError> {
    let Some(entry) = registry.entry_by_uuid(reference.guid()) else {
        let Some(candidate) = entry_by_hint(registry, sources, reference)? else {
            return Err(ReferenceResolutionError::Dangling {
                guid: reference.guid(),
                path: reference.path_hint().to_string(),
            });
        };
        return Err(ReferenceResolutionError::PathOccupiedCandidate {
            guid: reference.guid(),
            path: reference.path_hint().to_string(),
            candidate_uuid: candidate.uuid(),
            candidate_path: candidate.path().clone(),
        });
    };

    if entry.path().label() != reference.sub() {
        if let (None, Some(subasset)) = (entry.path().label(), reference.sub()) {
            // Only the GUID's source can diagnose a missing label; the hint may be stale or occupied.
            let labeled_locator = AssetUri::new(
                entry.path().scheme(),
                entry.path().path().to_owned(),
                Some(subasset.to_owned()),
            )
            .map_err(|error| ReferenceResolutionError::Registry {
                message: error.to_string(),
            })?;
            if registry.entry_by_path(&labeled_locator).is_none() {
                return Err(missing_subasset_error(
                    registry,
                    entry.path(),
                    reference,
                    subasset,
                ));
            }
        }
        // An exact labeled entry cannot replace the stable parent GUID.
        return Err(ReferenceResolutionError::Conflict {
            guid: reference.guid(),
            path: reference.path_hint().to_string(),
        });
    };

    let resolved_ref = AssetRef::try_new(
        entry.uuid(),
        sources.project_hint_for_locator(entry.path())?,
        entry.path().label().map(str::to_owned),
    )
    .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
    Ok(ResolvedProjectReference {
        reference: AssetReference::new(entry.uuid(), entry.path().clone()),
        repair: repair_between(reference, &resolved_ref),
    })
}

fn repair_between(stale: &AssetRef, resolved: &AssetRef) -> Option<ReferenceRepair> {
    if stale == resolved {
        return None;
    }
    debug_assert_eq!(stale.guid(), resolved.guid());
    debug_assert_eq!(stale.sub(), resolved.sub());
    Some(ReferenceRepair {
        stale: stale.clone(),
        resolved: resolved.clone(),
        kind: ReferenceRepairKind::PathHint,
    })
}

fn entry_by_hint<'a>(
    registry: &'a AssetRegistryIndex,
    sources: &impl ProjectSourceLookup,
    reference: &AssetRef,
) -> Result<Option<&'a AssetRegistryEntry>, ReferenceResolutionError> {
    let Some(base_locator) = sources.locator_for_project_hint(reference.path_hint())? else {
        return Ok(None);
    };
    let base_entry = registry.entry_by_path(&base_locator);
    let Some(subasset) = reference.sub() else {
        return Ok(base_entry);
    };

    let labeled_locator_text = format!("{base_locator}#{subasset}");
    let labeled_locator = AssetUri::parse(&labeled_locator_text).map_err(|error| {
        ReferenceResolutionError::Registry {
            message: error.to_string(),
        }
    })?;
    if let Some(entry) = registry.entry_by_path(&labeled_locator) {
        return Ok(Some(entry));
    }

    Err(missing_subasset_error(
        registry,
        &base_locator,
        reference,
        subasset,
    ))
}

fn missing_subasset_error(
    registry: &AssetRegistryIndex,
    base_locator: &AssetUri,
    reference: &AssetRef,
    subasset: &str,
) -> ReferenceResolutionError {
    let mut candidates = registry
        .source_entries(base_locator)
        .into_iter()
        .filter(|entry| entry.path().label().is_some())
        .map(|entry| entry.path().clone())
        .collect::<Vec<_>>();
    candidates.sort();
    ReferenceResolutionError::DanglingSubasset {
        guid: reference.guid(),
        path: reference.path_hint().to_string(),
        label: subasset.to_owned(),
        candidates,
    }
}

fn filesystem_locator_for_project_hint(
    roots: &[(RelPath, PathBuf)],
    hint: &RelPath,
) -> Result<Option<AssetUri>, ReferenceResolutionError> {
    let mut candidates = Vec::with_capacity(roots.len());
    for (root_rel, root) in roots {
        let Some(relative) = hint
            .as_str()
            .strip_prefix(root_rel.as_str())
            .and_then(|relative| relative.strip_prefix('/'))
        else {
            continue;
        };
        let path = root.join(relative);
        let locator = if let Some(locator) = logical_locator_for_persisted_source(root, &path)
            .map_err(|source| ReferenceResolutionError::PathIo {
                path: path.clone(),
                source,
            })? {
            Some(locator)
        } else if is_safe_regular_file(root, &path).map_err(|source| {
            ReferenceResolutionError::PathIo {
                path: path.clone(),
                source,
            }
        })? {
            Some(
                AssetUri::parse(&format!("res://{relative}")).map_err(|error| {
                    ReferenceResolutionError::Registry {
                        message: error.to_string(),
                    }
                })?,
            )
        } else {
            None
        };
        if let Some(locator) = locator {
            candidates.push(locator);
        }
    }
    match candidates.as_slice() {
        [] => Ok(None),
        [locator] => Ok(Some(locator.clone())),
        _ => Err(ReferenceResolutionError::AmbiguousPath {
            path: hint.to_string(),
        }),
    }
}

fn filesystem_project_hint_for_locator(
    roots: &[(RelPath, PathBuf)],
    locator: &AssetUri,
) -> Result<RelPath, ReferenceResolutionError> {
    let mut candidates = Vec::with_capacity(roots.len());
    for candidate @ (_, root) in roots {
        let path = persisted_source_path_for_locator(root, locator).map_err(|source| {
            ReferenceResolutionError::PathIo {
                path: root.join(locator.path()),
                source,
            }
        })?;
        if let Some(path) = path {
            candidates.push((candidate, path));
        }
    }
    let ((root_rel, root), path) = match candidates.as_slice() {
        [(candidate, path)] => (*candidate, path),
        [] => {
            return Err(ReferenceResolutionError::MissingPath {
                path: locator.to_string(),
            });
        }
        _ => {
            return Err(ReferenceResolutionError::AmbiguousPath {
                path: locator.to_string(),
            });
        }
    };
    let relative = path
        .strip_prefix(root)
        .map_err(|error| ReferenceResolutionError::Registry {
            message: format!(
                "persisted source {} escaped root {}: {error}",
                path.display(),
                root.display()
            ),
        })?;
    RelPath::parse(format!(
        "{}/{}",
        root_rel.as_str(),
        relative.to_string_lossy()
    ))
    .map_err(|source| ReferenceResolutionError::Path {
        path: locator.to_string(),
        source,
    })
}

#[cfg(test)]
#[path = "tests/reference_resolver.rs"]
mod tests;

#[cfg(test)]
#[path = "reference_resolver/tests/optimization_batch_is_runtime631_tests.rs"]
mod optimization_batch_is_runtime631_tests;
