//! Immutable manager rows and generation-paired catalog snapshots.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::catalog_snapshot::EditorPluginCatalogSnapshot;
use super::super::extension_catalog_report::EditorExtensionCatalogReport;
use super::{
    build_active_extensions, normalize_entries_for_loading_phase, EditorPluginDiscovery,
    EditorPluginSource, EditorPluginState,
};

/// One lightweight manager row. Descriptor and capability data stay in the catalog snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorPluginManagerEntry {
    pub(super) package_id: String,
    pub(super) source: EditorPluginSource,
    pub(super) loading_phase: super::super::phases::EditorPluginLoadingPhase,
    pub(super) state: EditorPluginState,
}

impl EditorPluginManagerEntry {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn source(&self) -> EditorPluginSource {
        self.source
    }

    pub fn loading_phase(&self) -> super::super::phases::EditorPluginLoadingPhase {
        self.loading_phase
    }

    pub fn state(&self) -> EditorPluginState {
        self.state
    }
}

/// Immutable manager read model paired with exactly one catalog generation.
#[derive(Clone, Debug)]
pub struct EditorPluginManagerSnapshot {
    generation: u64,
    catalog: Arc<EditorPluginCatalogSnapshot>,
    entries: Vec<EditorPluginManagerEntry>,
    pub(super) reached_loading_phase: Option<super::super::phases::EditorPluginLoadingPhase>,
    active_extensions: Arc<EditorExtensionCatalogReport>,
}

impl EditorPluginManagerSnapshot {
    pub(super) fn from_catalog(
        generation: u64,
        catalog: Arc<EditorPluginCatalogSnapshot>,
        previous_entries: &[EditorPluginManagerEntry],
        discoveries: &BTreeMap<String, EditorPluginDiscovery>,
        reached_loading_phase: Option<super::super::phases::EditorPluginLoadingPhase>,
    ) -> Self {
        let previous_by_package = previous_entries
            .iter()
            .map(|entry| (entry.package_id.as_str(), entry))
            .collect::<BTreeMap<_, _>>();
        let mut entries = catalog
            .package_manifests()
            .iter()
            .map(|package| {
                let mut entry = previous_by_package
                    .get(package.id.as_str())
                    .copied()
                    .cloned()
                    .unwrap_or_else(|| EditorPluginManagerEntry {
                        package_id: package.id.clone(),
                        source: discoveries
                            .get(package.id.as_str())
                            .map(EditorPluginDiscovery::source)
                            .unwrap_or(EditorPluginSource::Builtin),
                        loading_phase: discoveries
                            .get(package.id.as_str())
                            .map(EditorPluginDiscovery::loading_phase)
                            .unwrap_or(super::super::phases::EditorPluginLoadingPhase::Default),
                        // Admission has succeeded, but extension contributions stay inactive
                        // until the manager reaches this package's loading phase.
                        state: EditorPluginState::Validated,
                    });
                if catalog.is_package_faulted(package.id.as_str()) {
                    entry.state = EditorPluginState::Faulted;
                }
                if let Some(discovery) = discoveries.get(package.id.as_str()) {
                    entry.source = discovery.source();
                    entry.loading_phase = discovery.loading_phase();
                }
                entry
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));
        normalize_entries_for_loading_phase(&mut entries, reached_loading_phase);
        Self::from_parts(generation, catalog, entries, reached_loading_phase)
    }

    pub(super) fn from_parts(
        generation: u64,
        catalog: Arc<EditorPluginCatalogSnapshot>,
        entries: Vec<EditorPluginManagerEntry>,
        reached_loading_phase: Option<super::super::phases::EditorPluginLoadingPhase>,
    ) -> Self {
        let active_extensions = build_active_extensions(&catalog, &entries, generation);
        Self {
            generation,
            catalog,
            entries,
            reached_loading_phase,
            active_extensions,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn catalog_generation(&self) -> u64 {
        self.catalog.generation()
    }

    pub fn catalog_snapshot(&self) -> &Arc<EditorPluginCatalogSnapshot> {
        &self.catalog
    }

    pub fn entries(&self) -> &[EditorPluginManagerEntry] {
        &self.entries
    }

    /// The latest startup phase whose eligible entries have been activated.
    pub fn reached_loading_phase(&self) -> Option<super::super::phases::EditorPluginLoadingPhase> {
        self.reached_loading_phase
    }

    /// Extension contributions materialized for active entries in this manager generation only.
    pub fn active_extensions(&self) -> &Arc<EditorExtensionCatalogReport> {
        &self.active_extensions
    }

    pub fn entry(&self, package_id: &str) -> Option<&EditorPluginManagerEntry> {
        self.entries
            .binary_search_by(|entry| entry.package_id.as_str().cmp(package_id))
            .ok()
            .and_then(|index| self.entries.get(index))
    }
}
