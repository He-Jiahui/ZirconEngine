use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::{AssetRef, RelPath};

use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::safe_project_path::is_safe_regular_file;
use crate::asset::{AssetReference, AssetUri, AssetUuid, ReferenceResolutionError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRepairKind {
    Guid,
    PathHint,
    Subasset,
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

pub(crate) fn resolve_project_reference(
    registry: &AssetRegistryIndex,
    roots: &[(RelPath, PathBuf)],
    reference: &AssetRef,
) -> Result<ResolvedProjectReference, ReferenceResolutionError> {
    let entry = if let Some(entry) = registry.entry_by_uuid(reference.guid()) {
        let resolved = project_hint_for_entry(roots, entry)?;
        let resolved = AssetRef::try_new(
            entry.uuid(),
            resolved,
            entry.path().label().map(str::to_owned),
        )
        .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
        let repair = repair_between(reference, &resolved);
        return Ok(ResolvedProjectReference {
            reference: AssetReference::new(entry.uuid(), entry.path().clone()),
            repair,
        });
    } else {
        let Some(entry) = entry_by_hint(registry, roots, reference)? else {
            return Err(ReferenceResolutionError::Dangling {
                guid: reference.guid(),
                path: reference.path_hint().to_string(),
            });
        };
        entry
    };
    let resolved_ref = AssetRef::try_new(
        entry.uuid(),
        project_hint_for_entry(roots, entry)?,
        entry.path().label().map(str::to_owned),
    )
    .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
    Ok(ResolvedProjectReference {
        reference: AssetReference::new(entry.uuid(), entry.path().clone()),
        repair: Some(ReferenceRepair {
            stale: reference.clone(),
            resolved: resolved_ref,
            kind: ReferenceRepairKind::Guid,
        }),
    })
}

fn repair_between(stale: &AssetRef, resolved: &AssetRef) -> Option<ReferenceRepair> {
    if stale == resolved {
        return None;
    }
    let kind = if stale.guid() != resolved.guid() {
        ReferenceRepairKind::Guid
    } else if stale.path_hint() != resolved.path_hint() {
        ReferenceRepairKind::PathHint
    } else {
        ReferenceRepairKind::Subasset
    };
    Some(ReferenceRepair {
        stale: stale.clone(),
        resolved: resolved.clone(),
        kind,
    })
}

fn entry_by_hint<'a>(
    registry: &'a AssetRegistryIndex,
    roots: &[(RelPath, PathBuf)],
    reference: &AssetRef,
) -> Result<Option<&'a AssetRegistryEntry>, ReferenceResolutionError> {
    let hint = reference.path_hint().as_str();
    let mut candidates = Vec::new();
    for (root_rel, root) in roots {
        let Some(relative) = hint
            .strip_prefix(root_rel.as_str())
            .and_then(|relative| relative.strip_prefix('/'))
        else {
            continue;
        };
        let path = root.join(relative);
        if is_safe_regular_file(root, &path).map_err(|source| ReferenceResolutionError::PathIo {
            path: path.clone(),
            source,
        })? {
            candidates.push(relative);
        }
    }
    let relative = match candidates.as_slice() {
        [] => return Ok(None),
        [relative] => *relative,
        _ => {
            return Err(ReferenceResolutionError::AmbiguousPath {
                path: hint.to_owned(),
            });
        }
    };
    let base_locator_text = format!("res://{relative}");
    let base_locator = AssetUri::parse(&base_locator_text).map_err(|error| {
        ReferenceResolutionError::Registry {
            message: error.to_string(),
        }
    })?;
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
    Ok(registry.entry_by_path(&labeled_locator).or(base_entry))
}

fn project_hint_for_entry(
    roots: &[(RelPath, PathBuf)],
    entry: &AssetRegistryEntry,
) -> Result<RelPath, ReferenceResolutionError> {
    let mut candidates = Vec::new();
    for candidate @ (_, root) in roots {
        let path = root.join(entry.path().path());
        if is_safe_regular_file(root, &path).map_err(|source| ReferenceResolutionError::PathIo {
            path: path.clone(),
            source,
        })? {
            candidates.push(candidate);
        }
    }
    let (root, _) = match candidates.as_slice() {
        [candidate] => *candidate,
        [] => {
            return Err(ReferenceResolutionError::MissingPath {
                path: entry.path().to_string(),
            });
        }
        _ => {
            return Err(ReferenceResolutionError::AmbiguousPath {
                path: entry.path().to_string(),
            });
        }
    };
    RelPath::parse(format!("{}/{}", root.as_str(), entry.path().path())).map_err(|source| {
        ReferenceResolutionError::Path {
            path: entry.path().to_string(),
            source,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use zircon_runtime_interface::project::AssetRef;

    use super::*;
    use crate::asset::registry::AssetRegistryEntry;
    use crate::asset::AssetKind;

    #[test]
    fn resolution_reports_guid_path_repair_dangling_and_conflict_states() {
        let root = std::env::temp_dir().join(format!(
            "zircon_reference_resolution_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("models")).unwrap();
        fs::write(root.join("models/a.glb"), "a").unwrap();
        fs::write(root.join("models/b.glb"), "b").unwrap();
        let a: AssetUuid = "e1111111-2222-4333-8444-555555555555".parse().unwrap();
        let b: AssetUuid = "e2111111-2222-4333-8444-555555555555".parse().unwrap();
        let missing: AssetUuid = "e3111111-2222-4333-8444-555555555555".parse().unwrap();
        let registry = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(
                a,
                AssetUri::parse("res://models/a.glb").unwrap(),
                AssetKind::Model,
                "a",
            ),
            AssetRegistryEntry::new(
                b,
                AssetUri::parse("res://models/b.glb").unwrap(),
                AssetKind::Model,
                "b",
            ),
        ])
        .unwrap();
        let roots = vec![(RelPath::parse("assets").unwrap(), root.clone())];

        let exact =
            AssetRef::try_new(a, RelPath::parse("assets/models/a.glb").unwrap(), None).unwrap();
        assert_eq!(
            resolve_project_reference(&registry, &roots, &exact)
                .unwrap()
                .repair,
            None
        );

        let stale_guid = AssetRef::try_new(
            missing,
            RelPath::parse("assets/models/a.glb").unwrap(),
            None,
        )
        .unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &stale_guid)
                .unwrap()
                .repair,
            Some(ReferenceRepair { kind: ReferenceRepairKind::Guid, resolved, .. })
                if resolved.guid() == a
        ));

        let stale_subasset = AssetRef::try_new(
            missing,
            RelPath::parse("assets/models/a.glb").unwrap(),
            Some("MissingMesh".to_owned()),
        )
        .unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &stale_subasset)
                .unwrap()
                .repair,
            Some(ReferenceRepair {
                kind: ReferenceRepairKind::Guid,
                stale,
                resolved,
            }) if stale.sub() == Some("MissingMesh")
                && resolved.guid() == a
                && resolved.sub().is_none()
        ));

        let stale_path =
            AssetRef::try_new(a, RelPath::parse("assets/models/moved.glb").unwrap(), None).unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &stale_path)
                .unwrap()
                .repair,
            Some(ReferenceRepair {
                kind: ReferenceRepairKind::PathHint,
                ..
            })
        ));

        let dangling = AssetRef::try_new(
            missing,
            RelPath::parse("assets/models/missing.glb").unwrap(),
            None,
        )
        .unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &dangling),
            Err(ReferenceResolutionError::Dangling { .. })
        ));

        let occupied_hint =
            AssetRef::try_new(a, RelPath::parse("assets/models/b.glb").unwrap(), None).unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &occupied_hint)
                .unwrap()
                .repair,
            Some(ReferenceRepair { kind: ReferenceRepairKind::PathHint, resolved, .. })
                if resolved.path_hint().as_str() == "assets/models/a.glb"
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
