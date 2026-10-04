use zircon_runtime::plugin::PluginPackageManifest;

use super::{EditorPluginCatalog, EditorPluginCatalogStore};
use crate::core::plugin::EditorPluginDescriptor;

#[test]
fn snapshot_indexes_capabilities_to_sorted_package_ids() {
    let catalog = EditorPluginCatalog::from_descriptors(
        vec![
            EditorPluginDescriptor::new("plugin.zeta", "Zeta", "zeta").with_capability("shared"),
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
