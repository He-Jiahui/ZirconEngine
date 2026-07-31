//! Immutable snapshot publication tests for the plugin manager.

use std::sync::Arc;

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};

use super::super::{EditorPluginCatalog, EditorPluginManager, EditorPluginState};

struct RejectingPlugin {
    descriptor: EditorPluginDescriptor,
}

impl EditorPlugin for RejectingPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        _registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        Err(EditorExtensionRegistryError::View(
            "fixture rejected contribution".to_string(),
        ))
    }
}

#[test]
fn stable_reads_share_the_published_catalog_snapshot() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::default())
        .expect("an empty catalog is admissible");

    let first = manager.catalog_snapshot();
    let second = manager.catalog_snapshot();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.generation(), 1);
}

#[test]
fn manager_owns_the_initial_generation_for_a_multi_plugin_catalog() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_descriptors(
        vec![
            EditorPluginDescriptor::new("plugin.alpha", "Alpha", "alpha"),
            EditorPluginDescriptor::new("plugin.beta", "Beta", "beta"),
        ],
        Vec::<PluginPackageManifest>::new(),
    ))
    .expect("the multi-plugin catalog is admissible");

    assert_eq!(manager.catalog_snapshot().generation(), 1);
    assert_eq!(manager.state_snapshot().catalog_generation(), 1);
}

#[test]
fn publishing_replaces_the_generation_without_invalidating_existing_readers() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::default())
        .expect("an empty catalog is admissible");
    let previous = manager.catalog_snapshot();
    let published = manager
        .publish_catalog(EditorPluginCatalog::default())
        .expect("an empty replacement catalog is admissible");

    assert_eq!(previous.generation(), 1);
    assert_eq!(published.generation(), 2);
    assert!(!Arc::ptr_eq(&previous, &published));
    assert!(Arc::ptr_eq(&published, &manager.catalog_snapshot()));
}

#[test]
fn toggling_a_plugin_replaces_only_the_manager_state_generation() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![EditorPluginDescriptor::new(
            "plugin.sample",
            "Sample",
            "sample",
        )],
        Vec::<PluginPackageManifest>::new(),
    );
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");
    let previous = manager.state_snapshot();

    let updated = manager
        .set_enabled("plugin.sample", false)
        .expect("registered plugin should have a manager entry");

    assert_eq!(previous.generation(), 1);
    assert_eq!(updated.generation(), 2);
    assert_eq!(previous.catalog_generation(), updated.catalog_generation());
    assert_eq!(
        previous.active_extensions().active_manager_generation,
        Some(previous.generation())
    );
    assert_eq!(
        updated.active_extensions().active_manager_generation,
        Some(updated.generation())
    );
    assert_eq!(
        previous.entry("plugin.sample").map(|entry| entry.state()),
        Some(EditorPluginState::Validated)
    );
    assert_eq!(
        updated.entry("plugin.sample").map(|entry| entry.state()),
        Some(EditorPluginState::Disabled)
    );
}

#[test]
fn failed_registration_publishes_a_faulted_manager_entry() {
    let plugin = Arc::new(RejectingPlugin {
        descriptor: EditorPluginDescriptor::new("plugin.rejected", "Rejected", "rejected"),
    });
    let catalog = EditorPluginCatalog::from_plugins([(
        Arc::clone(&plugin) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new("plugin.rejected", "Rejected"),
    )]);
    let manager = EditorPluginManager::new(catalog).expect("the fixture catalog is admissible");

    assert_eq!(
        manager
            .state_snapshot()
            .entry("plugin.rejected")
            .map(|entry| entry.state()),
        Some(EditorPluginState::Faulted)
    );
}
