use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RetiredAssetReference};
use zircon_runtime_interface::resource::AssetReference;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::ReferenceResolutionError;
use crate::asset::reference_resolver::resolve_project_reference_from_lookup;
use crate::asset::registry::AssetRegistryIndex;

use super::AssetMigrationIssueKind;
use super::resolver_index::MigrationResolverIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionFailure {
    pub(super) kind: AssetMigrationIssueKind,
    pub(super) message: String,
}

pub(super) struct MigrationResolver<'a> {
    index: &'a AssetRegistryIndex,
    sources: &'a MigrationResolverIndex,
}

impl<'a> MigrationResolver<'a> {
    pub(super) fn new(index: &'a AssetRegistryIndex, sources: &'a MigrationResolverIndex) -> Self {
        Self { index, sources }
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
                ));
            }
            (None, Some(by_path)) => by_path,
            (Some(by_guid), None) => by_guid,
            (Some(by_guid), Some(_)) => by_guid,
        };
        let path_hint = self
            .sources
            .project_hint_for_locator(entry.path())
            .map_err(resolution_failure)?;
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
        resolve_project_reference_from_lookup(self.index, self.sources, reference)
            .map(|resolved| resolved.reference)
    }

    pub(super) fn repair_current(
        &self,
        reference: &AssetRef,
    ) -> Result<Option<AssetRef>, ResolutionFailure> {
        let resolved = resolve_project_reference_from_lookup(self.index, self.sources, reference)
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
