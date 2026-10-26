use std::sync::Arc;

use zircon_runtime::asset::registry::AssetRegistryIndex;
use zircon_runtime::asset::AssetUuid;
use zircon_runtime_interface::project::AssetRef;

use crate::core::asset::{AssetSourceAuthority, AssetSourceWritePolicy};
use crate::ui::host::editor_asset_manager::{
    EditorAssetDependencyCoverage, EditorAssetDependencyObservation, EditorAssetReadFacts,
    EditorAssetReadMissing, EditorAssetReadObservation, EditorAssetReadScope,
    EditorAssetReadSnapshot, EditorAssetReadStale, EditorAssetReadUnknown,
    EditorAssetRegistryReference,
};

use super::{lock_editor_asset_gate_recovering_poison, DefaultEditorAssetManager};

impl DefaultEditorAssetManager {
    pub(super) fn readonly_asset_snapshot_record(
        &self,
        reference: &AssetRef,
        expected: Option<&EditorAssetReadScope>,
        source_policy: Option<AssetSourceWritePolicy>,
    ) -> EditorAssetReadSnapshot {
        // Match the existing publisher's order. All work below is memory-only;
        // RAII releases every guard before the owned result reaches any caller.
        let _publish = lock_editor_asset_gate_recovering_poison(self.publish_gate.as_ref());
        let state = self.read_state_recovering_poison();
        let Some(project) = state.project.as_ref() else {
            return match expected {
                Some(_) => snapshot(
                    reference,
                    None,
                    EditorAssetReadObservation::Stale(EditorAssetReadStale::ProjectRetired),
                ),
                None => EditorAssetReadSnapshot::unknown(
                    reference,
                    EditorAssetReadUnknown::NoActiveProject,
                ),
            };
        };
        let Some(index) = state.asset_index.as_ref() else {
            return EditorAssetReadSnapshot::unknown(
                reference,
                EditorAssetReadUnknown::IndexUnavailable,
            );
        };
        let index = index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if index.project_activation_identity() != Some(project.activation_identity()) {
            return snapshot(
                reference,
                None,
                EditorAssetReadObservation::Stale(EditorAssetReadStale::ActivationMismatch),
            );
        }
        let Some(input) = index.catalog_input_generation() else {
            return EditorAssetReadSnapshot::unknown(
                reference,
                EditorAssetReadUnknown::InputUnavailable,
            );
        };
        if !Arc::ptr_eq(input, &project.catalog_input_generation()) {
            return snapshot(
                reference,
                None,
                EditorAssetReadObservation::Stale(EditorAssetReadStale::InputMismatch),
            );
        }
        let registry = index.runtime_registry();
        if !Arc::ptr_eq(registry, &project.asset_registry_shared()) {
            return snapshot(
                reference,
                None,
                EditorAssetReadObservation::Stale(EditorAssetReadStale::RegistryMismatch),
            );
        }
        let scope = EditorAssetReadScope::new(
            project.activation_identity().clone(),
            Arc::clone(input),
            Arc::clone(&state.catalog_generation),
        );
        if expected.is_some_and(|expected| !scope.same_publication(expected)) {
            return snapshot(
                reference,
                Some(scope),
                EditorAssetReadObservation::Stale(
                    EditorAssetReadStale::ExpectedPublicationMismatch,
                ),
            );
        }

        // Never resolve by the movable hint or turn a missing GUID's empty query
        // result into a known asset with zero dependencies.
        let Some(entry) = registry.entry_by_uuid(reference.guid()) else {
            return snapshot(
                reference,
                Some(scope),
                EditorAssetReadObservation::Missing(EditorAssetReadMissing::GuidNotRegistered),
            );
        };
        if entry.path().label() != reference.sub() {
            let registered_sub = entry.path().label().map(str::to_owned);
            return snapshot(
                reference,
                Some(scope),
                EditorAssetReadObservation::Missing(EditorAssetReadMissing::GuidSubMismatch {
                    registered_sub,
                }),
            );
        }

        let uuid = reference.guid().to_string();
        let asset = scope.catalog().asset_shared(&uuid);
        let details = scope.catalog().details(&uuid);
        if asset.as_ref().is_some_and(|asset| {
            // Both producers retain the original opaque source digest. Matching location/type
            // alone must not admit a previous source projection at this same identity.
            asset.locator != entry.path().to_string()
                || asset.kind != entry.type_marker()
                || asset.source_hash != entry.source_digest()
        }) || details
            .as_ref()
            .is_some_and(|details| asset.as_ref().map(Arc::as_ref) != Some(details.asset.as_ref()))
        {
            return snapshot(
                reference,
                Some(scope),
                EditorAssetReadObservation::Stale(EditorAssetReadStale::CatalogProjectionMismatch),
            );
        }
        let facts = EditorAssetReadFacts {
            registry_entry: entry.clone(),
            asset,
            details,
            dependencies: EditorAssetDependencyObservation {
                outgoing: registry
                    .get_dependencies_by_uuid(entry.uuid())
                    .into_iter()
                    .map(|uuid| registry_reference(registry, uuid))
                    .collect(),
                incoming: registry
                    .get_referencers_by_uuid(entry.uuid())
                    .into_iter()
                    .map(|uuid| registry_reference(registry, uuid))
                    .collect(),
                asset_edge_coverage: EditorAssetDependencyCoverage::Unknown,
                semantic_coverage: EditorAssetDependencyCoverage::Unknown,
            },
            source_policy_access: source_policy
                .map(|policy| AssetSourceAuthority::from_locator(policy, entry.path())),
        };
        snapshot(
            reference,
            Some(scope),
            EditorAssetReadObservation::Known(Box::new(facts)),
        )
    }
}

fn snapshot(
    reference: &AssetRef,
    scope: Option<EditorAssetReadScope>,
    observation: EditorAssetReadObservation,
) -> EditorAssetReadSnapshot {
    EditorAssetReadSnapshot {
        requested: reference.clone(),
        scope,
        observation,
    }
}

fn registry_reference(
    registry: &AssetRegistryIndex,
    uuid: AssetUuid,
) -> EditorAssetRegistryReference {
    match registry.entry_by_uuid(uuid) {
        Some(entry) => EditorAssetRegistryReference::Known(entry.clone()),
        None => EditorAssetRegistryReference::Missing(uuid),
    }
}

#[cfg(test)]
#[path = "tests/readonly_query.rs"]
mod tests;
