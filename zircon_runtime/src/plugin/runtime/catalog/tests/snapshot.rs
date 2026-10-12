use std::sync::Arc;

use super::{RuntimePluginCatalog, RuntimePluginCatalogSnapshot};

#[test]
fn snapshot_handle_shares_one_sealed_catalog_generation() {
    let catalog = RuntimePluginCatalog::from_descriptors([]);
    let generation = catalog.generation();
    let snapshot = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog));
    let cloned = Arc::clone(&snapshot);

    assert_eq!(snapshot.generation(), generation);
    assert!(Arc::ptr_eq(&snapshot, &cloned));
    assert_eq!(Arc::strong_count(&snapshot), 2);
}
