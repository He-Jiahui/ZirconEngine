use super::PluginCatalogGeneration;

#[test]
fn plugin_catalog_generation_keeps_one_word_layout() {
    assert_eq!(
        std::mem::size_of::<PluginCatalogGeneration>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::size_of::<Option<PluginCatalogGeneration>>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        PluginCatalogGeneration::INITIAL
            .checked_next()
            .expect("initial catalog generation should have a successor")
            .get(),
        2
    );
    assert!(PluginCatalogGeneration::from_raw_for_test(u64::MAX)
        .checked_next()
        .is_none());
}
