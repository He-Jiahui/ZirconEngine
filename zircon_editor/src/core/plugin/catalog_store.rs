//! Generation owner for immutable editor-plugin catalog snapshots.

use std::sync::{Arc, RwLock};

use super::catalog::EditorPluginCatalog;
use super::catalog_snapshot::EditorPluginCatalogSnapshot;

#[derive(Debug)]
pub(crate) struct EditorPluginCatalogStore {
    snapshot: RwLock<Arc<EditorPluginCatalogSnapshot>>,
}

impl EditorPluginCatalogStore {
    pub(super) fn new(catalog: EditorPluginCatalog) -> Self {
        Self {
            snapshot: RwLock::new(Arc::new(EditorPluginCatalogSnapshot::from_catalog(
                1, catalog,
            ))),
        }
    }

    pub(super) fn snapshot(&self) -> Arc<EditorPluginCatalogSnapshot> {
        Arc::clone(
            &self
                .snapshot
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Reserves the next catalog generation for an already materialized manager candidate.
    pub(super) fn next_generation(&self) -> u64 {
        self.snapshot().generation().saturating_add(1)
    }

    /// Publishes a candidate prepared by the manager's serialized lifecycle transaction.
    pub(super) fn publish_prepared(
        &self,
        snapshot: Arc<EditorPluginCatalogSnapshot>,
    ) -> Arc<EditorPluginCatalogSnapshot> {
        let mut snapshot_slot = self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(
            snapshot.generation(),
            snapshot_slot.generation().saturating_add(1),
            "manager must publish exactly the next catalog generation"
        );
        *snapshot_slot = Arc::clone(&snapshot);
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::PluginPackageManifest;

    use super::{EditorPluginCatalog, EditorPluginCatalogStore};
    use crate::core::plugin::EditorPluginDescriptor;

    #[test]
    fn snapshot_indexes_capabilities_to_sorted_package_ids() {
        let catalog = EditorPluginCatalog::from_descriptors(
            vec![
                EditorPluginDescriptor::new("plugin.zeta", "Zeta", "zeta")
                    .with_capability("shared"),
                EditorPluginDescriptor::new("plugin.alpha", "Alpha", "alpha")
                    .with_capability("shared")
                    .with_capability("alpha-only"),
            ],
            Vec::<PluginPackageManifest>::new(),
        );
        let snapshot = EditorPluginCatalogStore::new(catalog).snapshot();

        assert_eq!(
            snapshot.packages_for_capability("shared"),
            &["plugin.alpha".to_string(), "plugin.zeta".to_string()]
        );
        assert_eq!(
            snapshot.packages_for_capability("missing"),
            &[] as &[String]
        );
    }

    #[test]
    fn projection_preserves_editor_registration_capabilities() {
        let catalog = EditorPluginCatalog::from_descriptors(
            vec![
                EditorPluginDescriptor::new("plugin.sample", "Sample", "sample")
                    .with_capability("editor.command"),
            ],
            vec![PluginPackageManifest::new(
                "plugin.sample",
                "Runtime package",
            )],
        );
        let snapshot = EditorPluginCatalogStore::new(catalog).snapshot();
        let entry = snapshot
            .projection()
            .entries()
            .first()
            .expect("registered package should have a projection entry");

        assert_eq!(entry.crate_name, "sample");
        assert_eq!(entry.capabilities, vec!["editor.command".to_string()]);
    }
}
