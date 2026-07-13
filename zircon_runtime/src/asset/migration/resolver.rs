use std::path::PathBuf;

use zircon_runtime_interface::project::{
    AssetRef, PersistedAssetReference, RelPath, RetiredAssetReference,
};
use zircon_runtime_interface::resource::AssetReference;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::safe_project_path::is_safe_regular_file;
use crate::asset::ReferenceResolutionError;

use super::AssetMigrationIssueKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionFailure {
    pub(super) kind: AssetMigrationIssueKind,
    pub(super) message: String,
}

pub(super) struct MigrationResolver<'a> {
    index: &'a AssetRegistryIndex,
    roots: &'a [(RelPath, PathBuf)],
}

impl<'a> MigrationResolver<'a> {
    pub(super) fn new(index: &'a AssetRegistryIndex, roots: &'a [(RelPath, PathBuf)]) -> Self {
        Self { index, roots }
    }

    pub(super) fn resolve(
        &self,
        retired_reference: RetiredAssetReference,
    ) -> Result<AssetRef, ResolutionFailure> {
        if retired_reference.locator().scheme() != ResourceScheme::Res {
            return Err(failure(
                AssetMigrationIssueKind::UnsupportedScheme,
                format!(
                    "persistent project reference must use res://, found {}",
                    retired_reference.locator()
                ),
            ));
        }
        let by_guid = self.index.entry_by_uuid(retired_reference.guid());
        let by_path = self.index.entry_by_path(retired_reference.locator());
        let entry = match (by_guid, by_path) {
            (None, None) => {
                return Err(failure(
                    AssetMigrationIssueKind::DanglingReference,
                    format!(
                        "asset guid {} and path {} are both unregistered",
                        retired_reference.guid(),
                        retired_reference.locator()
                    ),
                ))
            }
            (None, Some(by_path)) => by_path,
            (Some(by_guid), None) => by_guid,
            (Some(by_guid), Some(_)) => by_guid,
        };
        let path_hint = self.project_relative_path(entry.path().path())?;
        AssetRef::try_new(
            entry.uuid(),
            path_hint,
            entry.path().label().map(str::to_string),
        )
        .map_err(|error| failure(AssetMigrationIssueKind::InvalidDocument, error.to_string()))
    }

    pub(super) fn resolve_current(
        &self,
        reference: &AssetRef,
    ) -> Result<AssetReference, ReferenceResolutionError> {
        crate::asset::resolve_project_reference(self.index, self.roots, reference)
            .map(|resolved| resolved.reference)
    }

    pub(super) fn repair_current(
        &self,
        reference: &AssetRef,
    ) -> Result<Option<AssetRef>, ResolutionFailure> {
        let resolved = crate::asset::resolve_project_reference(self.index, self.roots, reference)
            .map_err(resolution_failure)?;
        let Some(repair) = resolved.repair else {
            return Ok(None);
        };
        Ok(Some(repair.resolved))
    }

    pub(super) fn resolve_persisted(
        &self,
        reference: &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError> {
        if let Some(reference) = reference.project_ref() {
            return self.resolve_current(reference);
        }
        let locator = reference
            .builtin_locator()
            .ok_or(ReferenceResolutionError::MissingPayload)?;
        if locator.scheme() != ResourceScheme::Builtin {
            return Err(ReferenceResolutionError::UnsupportedScheme {
                locator: locator.clone(),
            });
        }
        Ok(AssetReference::from_locator(locator.clone()))
    }

    fn project_relative_path(&self, relative: &str) -> Result<RelPath, ResolutionFailure> {
        let mut candidates = Vec::new();
        for candidate @ (_, root) in self.roots {
            let path = root.join(relative);
            match is_safe_regular_file(root, &path) {
                Ok(true) => candidates.push(candidate),
                Ok(false) => {}
                Err(error) => {
                    return Err(failure(
                        AssetMigrationIssueKind::PathIo,
                        format!("failed to inspect {}: {error}", path.display()),
                    ));
                }
            }
        }
        let (root_rel, _) = match candidates.as_slice() {
            [candidate] => *candidate,
            [] => {
                return Err(failure(
                    AssetMigrationIssueKind::MissingPath,
                    format!("registered source res://{relative} has no file in any asset root"),
                ))
            }
            _ => {
                return Err(failure(
                    AssetMigrationIssueKind::AmbiguousPath,
                    format!("registered source res://{relative} exists in multiple asset roots"),
                ))
            }
        };
        RelPath::parse(format!("{}/{relative}", root_rel.as_str()))
            .map_err(|error| failure(AssetMigrationIssueKind::UnsafePath, error.to_string()))
    }
}

fn failure(kind: AssetMigrationIssueKind, message: String) -> ResolutionFailure {
    ResolutionFailure { kind, message }
}

fn resolution_failure(error: ReferenceResolutionError) -> ResolutionFailure {
    let kind = match &error {
        ReferenceResolutionError::Dangling { .. }
        | ReferenceResolutionError::MissingGuid { .. } => {
            AssetMigrationIssueKind::DanglingReference
        }
        ReferenceResolutionError::MissingPath { .. } => AssetMigrationIssueKind::MissingPath,
        ReferenceResolutionError::AmbiguousPath { .. } => AssetMigrationIssueKind::AmbiguousPath,
        ReferenceResolutionError::Conflict { .. } | ReferenceResolutionError::Registry { .. } => {
            AssetMigrationIssueKind::RegistryConflict
        }
        ReferenceResolutionError::UnsupportedScheme { .. } => {
            AssetMigrationIssueKind::UnsupportedScheme
        }
        ReferenceResolutionError::Path { .. } => AssetMigrationIssueKind::UnsafePath,
        ReferenceResolutionError::PathIo { .. } => AssetMigrationIssueKind::PathIo,
        _ => AssetMigrationIssueKind::InvalidDocument,
    };
    failure(kind, error.to_string())
}
