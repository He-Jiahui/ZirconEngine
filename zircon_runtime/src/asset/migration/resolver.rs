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
        // A matching GUID establishes the source identity and can repair a moved path. Legacy
        // compound references may carry the parent GUID plus a subasset label, so an unlabeled
        // parent entry is also a valid source hint. Other label mismatches must stay on the
        // retired locator so the shared resolver cannot fall back to the wrong subasset.
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;
    use zircon_runtime_interface::project::{
        migrate_retired_persisted_asset_reference_with, RelPath, RetiredAssetRefMigrationError,
    };

    use super::*;
    use crate::asset::migration::{MigrationResolverIndex, MigrationSourceProjection};
    use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    #[test]
    fn retired_migration_never_downgrades_missing_labeled_subasset_to_parent() {
        let parent: AssetUuid = "c1111111-2222-4333-8444-555555555555".parse().unwrap();
        let root = PathBuf::from("E:/migration-resolver-test");
        let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
            parent,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "hero-digest",
        )])
        .unwrap();
        let sources = MigrationResolverIndex::build(
            [MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                root.join("assets"),
                RelPath::parse("models/hero.glb").unwrap(),
                root.join("assets/models/hero.glb"),
            )],
            [],
        )
        .unwrap();
        let resolver = MigrationResolver::new(&registry, &sources);

        let error = migrate_retired_persisted_asset_reference_with(
            json!({
                "uuid": parent.to_string(),
                "url": "res://models/hero.glb#MissingMesh",
            }),
            |reference| resolver.resolve(reference),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            RetiredAssetRefMigrationError::Resolve(failure)
                if failure.kind == AssetMigrationIssueKind::DanglingReference
                    && failure.message.contains("missing subasset MissingMesh")
        ));
    }

    #[test]
    fn retired_migration_uses_a_matching_guid_to_repair_a_moved_source() {
        let guid: AssetUuid = "c3111111-2222-4333-8444-555555555555".parse().unwrap();
        let root = PathBuf::from("E:/migration-resolver-test");
        let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
            guid,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "hero-digest",
        )])
        .unwrap();
        let sources = MigrationResolverIndex::build(
            [MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                root.join("assets"),
                RelPath::parse("models/hero.glb").unwrap(),
                root.join("assets/models/hero.glb"),
            )],
            [],
        )
        .unwrap();
        let resolver = MigrationResolver::new(&registry, &sources);

        let migrated = migrate_retired_persisted_asset_reference_with(
            json!({
                "uuid": guid.to_string(),
                "url": "res://legacy/hero.glb",
            }),
            |reference| resolver.resolve(reference),
        )
        .unwrap();

        assert_eq!(
            migrated,
            json!({
                "kind": "project",
                "guid": guid.to_string(),
                "path_hint": "assets/models/hero.glb",
                "sub": null,
            })
        );
    }

    #[test]
    fn retired_migration_repairs_parent_guid_to_the_exact_labeled_subasset() {
        let parent: AssetUuid = "c4111111-2222-4333-8444-555555555555".parse().unwrap();
        let mesh: AssetUuid = "c5111111-2222-4333-8444-555555555555".parse().unwrap();
        let root = PathBuf::from("E:/migration-resolver-test");
        let registry = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(
                parent,
                AssetUri::parse("res://models/hero.glb").unwrap(),
                AssetKind::Model,
                "hero-digest",
            ),
            AssetRegistryEntry::new(
                mesh,
                AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
                AssetKind::Mesh,
                "mesh-digest",
            ),
        ])
        .unwrap();
        let sources = MigrationResolverIndex::build(
            [MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                root.join("assets"),
                RelPath::parse("models/hero.glb").unwrap(),
                root.join("assets/models/hero.glb"),
            )],
            [],
        )
        .unwrap();
        let resolver = MigrationResolver::new(&registry, &sources);

        let migrated = migrate_retired_persisted_asset_reference_with(
            json!({
                "uuid": parent.to_string(),
                "url": "res://models/hero.glb#Mesh0",
            }),
            |reference| resolver.resolve(reference),
        )
        .unwrap();

        assert_eq!(
            migrated,
            json!({
                "kind": "project",
                "guid": mesh.to_string(),
                "path_hint": "assets/models/hero.glb",
                "sub": "Mesh0",
            })
        );
    }

    #[test]
    fn retired_migration_repairs_moved_compound_source_from_parent_guid_and_label() {
        let parent: AssetUuid = "c6111111-2222-4333-8444-555555555555".parse().unwrap();
        let mesh: AssetUuid = "c7111111-2222-4333-8444-555555555555".parse().unwrap();
        let root = PathBuf::from("E:/migration-resolver-test");
        let registry = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(
                parent,
                AssetUri::parse("res://models/hero.glb").unwrap(),
                AssetKind::Model,
                "hero-digest",
            ),
            AssetRegistryEntry::new(
                mesh,
                AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
                AssetKind::Mesh,
                "mesh-digest",
            ),
        ])
        .unwrap();
        let sources = MigrationResolverIndex::build(
            [MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                root.join("assets"),
                RelPath::parse("models/hero.glb").unwrap(),
                root.join("assets/models/hero.glb"),
            )],
            [],
        )
        .unwrap();
        let resolver = MigrationResolver::new(&registry, &sources);

        let migrated = migrate_retired_persisted_asset_reference_with(
            json!({
                "uuid": parent.to_string(),
                "url": "res://legacy/hero.glb#Mesh0",
            }),
            |reference| resolver.resolve(reference),
        )
        .unwrap();

        assert_eq!(
            migrated,
            json!({
                "kind": "project",
                "guid": mesh.to_string(),
                "path_hint": "assets/models/hero.glb",
                "sub": "Mesh0",
            })
        );
    }
}
