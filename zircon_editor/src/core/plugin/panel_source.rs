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

    /// 从同代管理器行及目录投影按包身份取一行；调用者需重新构建 PanelSource 才能观察后续发布。
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
#[path = "tests/panel_source.rs"]
mod tests;
