use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RetiredAssetReference};
use zircon_runtime_interface::resource::AssetReference;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::reference_resolver::resolve_project_reference_from_lookup;
use crate::asset::registry::AssetRegistryIndex;
use crate::asset::ReferenceResolutionError;

use super::resolver_index::MigrationResolverIndex;
use super::AssetMigrationIssueKind;

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
        // A matching GUID can repair a moved source path. An unlabeled parent's source supplies
        // diagnostics for retired labeled references, but the shared resolver still rejects
        // their GUID/label mismatch instead of replacing the target identity.
        let locator_for_hint = self
            .index
            .entry_by_uuid(retired_reference.guid())
            .filter(|entry| {
                entry.path().label() == retired_reference.locator().label()
                    || (entry.path().label().is_none()
                        && retired_reference.locator().label().is_some())
            })
            .map(|entry| entry.path())
            .unwrap_or_else(|| retired_reference.locator());
        let path_hint = self
            .sources
            .project_hint_for_locator(locator_for_hint)
            .map_err(|error| match error {
                ReferenceResolutionError::MissingPath { .. } => failure(
                    AssetMigrationIssueKind::DanglingReference,
                    format!(
                        "asset guid {} and path {} are both unregistered",
                        retired_reference.guid(),
                        retired_reference.locator()
                    ),
                ),
                error => resolution_failure(error),
            })?;
        let reference = AssetRef::try_new(
            retired_reference.guid(),
            path_hint,
            retired_reference.locator().label().map(str::to_string),
        )
        .map_err(|error| failure(AssetMigrationIssueKind::InvalidDocument, error.to_string()))?;
        let resolved = resolve_project_reference_from_lookup(self.index, self.sources, &reference)
            .map_err(resolution_failure)?;
        Ok(resolved.repair.map_or(reference, |repair| repair.resolved))
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

    pub(super) fn resolver_index_lookups(&self) -> usize {
        self.sources.lookup_count()
    }
}

fn failure(kind: AssetMigrationIssueKind, message: String) -> ResolutionFailure {
    ResolutionFailure { kind, message }
}

fn resolution_failure(error: ReferenceResolutionError) -> ResolutionFailure {
    let kind = match &error {
        ReferenceResolutionError::Dangling { .. }
        | ReferenceResolutionError::DanglingSubasset { .. }
        | ReferenceResolutionError::MissingGuid { .. }
        | ReferenceResolutionError::PathOccupiedCandidate { .. } => {
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

#[cfg(test)]
#[path = "tests/resolver.rs"]
mod tests;
