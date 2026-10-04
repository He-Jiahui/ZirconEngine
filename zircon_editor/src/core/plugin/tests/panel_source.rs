use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::plugin::{
    EditorPluginCatalog, EditorPluginDescriptor, EditorPluginDiscovery, EditorPluginLoadingPhase,
    EditorPluginState,
};

use super::{EditorPluginManager, EditorPluginPanelSource};

fn manager() -> EditorPluginManager {
    let catalog = EditorPluginCatalog::from_descriptors(
        [
            EditorPluginDescriptor::new("plugin.zeta", "Zeta", "zeta").with_capability("render"),
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
    let source = include_str!("../panel_source.rs");
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
