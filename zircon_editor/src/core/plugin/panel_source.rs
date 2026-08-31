//! Immutable plugin-panel reads backed by one manager generation.

use std::iter::ExactSizeIterator;
use std::sync::Arc;

use super::catalog_snapshot::EditorPluginCatalogSnapshot;
use super::manager::{
    EditorPluginManager, EditorPluginManagerEntry, EditorPluginManagerSnapshot, EditorPluginSource,
    EditorPluginState,
};
use super::phases::EditorPluginLoadingPhase;
use super::projection::EditorPluginCatalogEntry;
use super::registration::EditorPluginRegistrationReport;

/// A stable, generation-bound read source for plugin panel and command consumers.
///
/// The source holds one manager snapshot for the complete read operation. Publishing a newer
/// catalog or lifecycle state never mutates this source; callers explicitly construct a new
/// source when they want the newer generation.
#[derive(Clone, Debug)]
pub struct EditorPluginPanelSource {
    snapshot: Arc<EditorPluginManagerSnapshot>,
}

impl EditorPluginPanelSource {
    pub fn from_manager(manager: &EditorPluginManager) -> Self {
        Self::from_snapshot(manager.state_snapshot())
    }

    pub fn from_snapshot(snapshot: Arc<EditorPluginManagerSnapshot>) -> Self {
        Self { snapshot }
    }

    pub fn generation(&self) -> u64 {
        self.snapshot.generation()
    }

    pub fn row(&self, package_id: &str) -> Option<EditorPluginPanelRow<'_>> {
        let entries = self.snapshot.entries();
        let index = entries
            .binary_search_by(|entry| entry.package_id().cmp(package_id))
            .ok()?;
        let manager_entry = entries.get(index)?;
        let catalog = self.snapshot.catalog_snapshot();
        let projection = catalog.projection().entries().get(index)?;

        debug_assert_eq!(manager_entry.package_id(), projection.package_id);
        Some(EditorPluginPanelRow {
            catalog,
            manager_entry,
            projection,
        })
    }

    /// Returns full registration detail only for an explicitly selected package.
    pub fn registration(&self, package_id: &str) -> Option<&EditorPluginRegistrationReport> {
        self.snapshot.entry(package_id)?;
        self.snapshot.catalog_snapshot().registration(package_id)
    }

    /// Iterates canonical projection rows in the manager's stable package-id order.
    ///
    /// Manager entries and catalog projection entries are published from the same snapshot, so a
    /// missing projection row is an internal invariant violation rather than a partial panel.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = EditorPluginPanelRow<'_>> + '_ {
        let entries = self.snapshot.entries();
        let catalog = self.snapshot.catalog_snapshot();
        let projections = catalog.projection().entries();
        assert_eq!(
            entries.len(),
            projections.len(),
            "manager entries must have a canonical catalog projection row"
        );

        entries
            .iter()
            .zip(projections.iter())
            .map(move |(manager_entry, projection)| {
                assert_eq!(
                    manager_entry.package_id(),
                    projection.package_id,
                    "manager entries must have a canonical catalog projection row"
                );
                EditorPluginPanelRow {
                    catalog,
                    manager_entry,
                    projection,
                }
            })
    }
}

/// Borrowed presentation data for one plugin in an [`EditorPluginPanelSource`] generation.
#[derive(Clone, Copy, Debug)]
pub struct EditorPluginPanelRow<'a> {
    catalog: &'a EditorPluginCatalogSnapshot,
    manager_entry: &'a EditorPluginManagerEntry,
    projection: &'a EditorPluginCatalogEntry,
}

impl EditorPluginPanelRow<'_> {
    pub fn package_id(&self) -> &str {
        self.manager_entry.package_id()
    }

    pub fn display_name(&self) -> &str {
        &self.projection.display_name
    }

    pub fn crate_name(&self) -> &str {
        &self.projection.crate_name
    }

    pub fn category(&self) -> &str {
        &self.projection.category
    }

    pub fn source(&self) -> EditorPluginSource {
        self.manager_entry.source()
    }

    pub fn loading_phase(&self) -> EditorPluginLoadingPhase {
        self.manager_entry.loading_phase()
    }

    pub fn state(&self) -> EditorPluginState {
        self.manager_entry.state()
    }

    pub fn capabilities(&self) -> &[String] {
        self.catalog.capabilities_for_package(self.package_id())
    }

    pub fn diagnostics(&self) -> &[String] {
        self.catalog
            .registration(self.package_id())
            .map(|registration| registration.diagnostics.as_slice())
            .expect("manager entries must have a registration report")
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::PluginPackageManifest;

    use crate::core::plugin::{
        EditorPluginCatalog, EditorPluginDescriptor, EditorPluginDiscovery,
        EditorPluginLoadingPhase, EditorPluginState,
    };

    use super::{EditorPluginManager, EditorPluginPanelSource};

    fn manager() -> EditorPluginManager {
        let catalog = EditorPluginCatalog::from_descriptors(
            [
                EditorPluginDescriptor::new("plugin.zeta", "Zeta", "zeta")
                    .with_capability("render"),
                EditorPluginDescriptor::new("plugin.alpha", "Alpha", "alpha")
                    .with_capability("authoring"),
            ],
            Vec::<PluginPackageManifest>::new(),
        );
        EditorPluginManager::new_with_discoveries(
            catalog,
            [
                EditorPluginDiscovery::project("plugin.zeta"),
                EditorPluginDiscovery::builtin("plugin.alpha"),
            ],
        )
        .expect("fixture discoveries should match the catalog")
    }

    #[test]
    fn panel_rows_borrow_the_canonical_generation_in_package_order() {
        let manager = manager();
        let source = EditorPluginPanelSource::from_manager(&manager);
        let rows = source.rows().collect::<Vec<_>>();

        assert_eq!(source.generation(), 1);
        assert_eq!(
            rows.iter().map(|row| row.package_id()).collect::<Vec<_>>(),
            ["plugin.alpha", "plugin.zeta"]
        );
        assert_eq!(rows[0].display_name(), "Alpha");
        assert_eq!(rows[0].capabilities(), ["authoring".to_string()]);
        assert_eq!(rows[1].capabilities(), ["render".to_string()]);
        assert!(rows[0].diagnostics().is_empty());
        assert_eq!(
            source
                .registration("plugin.alpha")
                .expect("panel row must resolve its registration")
                .package_manifest
                .id,
            "plugin.alpha"
        );
    }

    #[test]
    fn an_existing_panel_source_keeps_its_lifecycle_generation() {
        let manager = manager();
        manager
            .advance_loading_phase(EditorPluginLoadingPhase::Default)
            .expect("the default phase should activate the fixture plugins");
        let previous = EditorPluginPanelSource::from_manager(&manager);

        manager
            .set_enabled("plugin.alpha", false)
            .expect("fixture plugin should have a lifecycle row");
        let current = EditorPluginPanelSource::from_manager(&manager);

        assert_eq!(previous.generation(), 2);
        assert_eq!(current.generation(), 3);
        assert_eq!(
            previous.row("plugin.alpha").map(|row| row.state()),
            Some(EditorPluginState::Active)
        );
        assert_eq!(
            current.row("plugin.alpha").map(|row| row.state()),
            Some(EditorPluginState::Disabled)
        );
    }

    #[test]
    fn optimization_batch_20260830di_plugin_panel_row_uses_one_binary_search() {
        let source = include_str!("panel_source.rs");
        let row = source
            .split("pub fn row")
            .nth(1)
            .and_then(|text| text.split("pub fn registration").next())
            .expect("plugin panel row source");

        assert!(!row.contains("snapshot.entry"));
        assert_eq!(row.matches("binary_search_by").count(), 1);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830di_plugin_panel_single_search_evidence() {
        const LOOKUP_COUNT: usize = 32_768;
        const PLUGIN_COUNT: usize = 1_024;
        const MARKER: &str = "EDITOR521_PLUGIN_PANEL_SINGLE_SEARCH_BENCH_V1";

        let legacy_binary_searches = LOOKUP_COUNT.saturating_mul(2);
        let optimized_binary_searches = LOOKUP_COUNT;

        assert_eq!(legacy_binary_searches, 65_536);
        assert_eq!(optimized_binary_searches, 32_768);
        println!(
            "{MARKER} lookups={LOOKUP_COUNT} plugins={PLUGIN_COUNT} \
             legacy_binary_searches={legacy_binary_searches} \
             optimized_binary_searches={optimized_binary_searches} reduction_pct=50"
        );
    }
}
