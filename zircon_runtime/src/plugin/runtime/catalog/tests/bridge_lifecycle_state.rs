use std::sync::Arc;

use super::*;

#[test]
fn bridge_lifecycle_state_keeps_shared_catalog_and_extension_report_snapshots() {
    let catalog = RuntimePluginCatalog::from_descriptors([]);
    let extension_report = Arc::new(catalog.runtime_extensions());
    let snapshot = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog));

    let state = RuntimePluginBridgeLifecycleState::from_snapshot_and_extension_report(
        Arc::clone(&snapshot),
        Arc::clone(&extension_report),
    );
    let cloned = state.clone();

    assert!(Arc::ptr_eq(&snapshot, state.snapshot()));
    assert!(Arc::ptr_eq(&extension_report, &state.extension_report));
    assert!(Arc::ptr_eq(state.snapshot(), cloned.snapshot()));
    assert!(Arc::ptr_eq(
        &state.extension_report,
        &cloned.extension_report
    ));
}
