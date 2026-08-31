//! Atomic catalog publication shared by complete and project-scoped replacements.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::admission::validate_catalog_admission;
use super::discovery::{EditorPluginSource, discovery_index};
use super::snapshot::entries_for_catalog;
use super::state::activate_eligible_entries;
use super::{
    EditorPluginCatalog, EditorPluginCatalogSnapshot, EditorPluginDiscovery,
    EditorPluginDiscoveryError, EditorPluginManager, EditorPluginManagerEntry,
    EditorPluginManagerSnapshot, dispatch_hot_reloaded_replacements, replaced_live_package_ids,
    reset_replaced_active_entries, retire_replaced_active_entries,
};

impl EditorPluginManager {
    /// Atomically publishes a complete replacement catalog for the next generation.
    pub(crate) fn publish_catalog(
        &self,
        catalog: EditorPluginCatalog,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
        self.publish_catalog_with_discoveries(catalog, std::iter::empty())
    }

    /// Atomically publishes a replacement catalog together with its validated discovery metadata.
    pub(crate) fn publish_catalog_with_discoveries(
        &self,
        catalog: EditorPluginCatalog,
        discoveries: impl IntoIterator<Item = EditorPluginDiscovery>,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
        validate_catalog_admission(&catalog)?;
        let discoveries = discovery_index(&catalog, discoveries)?;
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginDiscoveryError::MutationInProgress)?;
        self.publish_catalog_with_indexed_discoveries(catalog, discoveries)
    }

    /// Publishes a candidate while the caller holds `lifecycle_mutation`.
    pub(super) fn publish_catalog_with_indexed_discoveries(
        &self,
        mut catalog: EditorPluginCatalog,
        discoveries: BTreeMap<String, EditorPluginDiscovery>,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
        let previous = self.state_snapshot();
        let mut entries = entries_for_catalog(
            &catalog,
            previous.entries(),
            &discoveries,
            previous.reached_loading_phase,
        );
        let mut previous_catalog = previous.catalog_snapshot().clone_catalog();
        if let Some(package_id) = active_package_retracted(&previous, &entries) {
            return Err(EditorPluginDiscoveryError::PhaseRetractionRequiresDisable { package_id });
        }
        let replaced_live_package_ids =
            replaced_live_package_ids(&previous, &previous_catalog, &catalog);
        let mut previous_entries = previous.entries().to_vec();
        if let Err(error) = retire_replaced_active_entries(
            &mut previous_catalog,
            &mut previous_entries,
            &replaced_live_package_ids,
        ) {
            self.publish_manager_snapshot(
                &previous,
                Some(previous_catalog),
                previous_entries,
                previous.reached_loading_phase,
            );
            return Err(error);
        }
        reset_replaced_active_entries(&previous_catalog, &catalog, &mut entries);
        activate_eligible_entries(&mut catalog, &mut entries, previous.reached_loading_phase);
        dispatch_hot_reloaded_replacements(&mut catalog, &mut entries, &replaced_live_package_ids);
        let snapshot = self.publish_manager_snapshot(
            &previous,
            Some(catalog),
            entries,
            previous.reached_loading_phase,
        );
        Ok(Arc::clone(snapshot.catalog_snapshot()))
    }
}

fn active_package_retracted(
    previous: &EditorPluginManagerSnapshot,
    candidate: &[EditorPluginManagerEntry],
) -> Option<String> {
    previous
        .entries()
        .iter()
        .filter(|entry| entry.state == super::EditorPluginState::Active)
        .filter(|entry| entry.source != EditorPluginSource::Project)
        .find_map(|active| {
            candidate
                .binary_search_by(|entry| entry.package_id.as_str().cmp(active.package_id()))
                .ok()
                .and_then(|index| candidate.get(index))
                .filter(|entry| entry.state == super::EditorPluginState::Active)
                .is_none()
                .then(|| active.package_id.clone())
        })
}
