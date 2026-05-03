const MODULE_WIRING: &str = include_str!("../mod.rs");

const DEFERRED_RENDERER_TEST_SOURCES: &[&str] = &[
    "virtual_geometry_args_source_authority",
    "virtual_geometry_execution_args_authority",
    "virtual_geometry_execution_stats",
    "virtual_geometry_gpu",
    "virtual_geometry_node_and_cluster_cull_execution",
    "virtual_geometry_prepare_render",
    "virtual_geometry_submission_authority",
    "virtual_geometry_submission_execution_order",
    "virtual_geometry_unified_indirect",
];

#[test]
fn deferred_renderer_test_sources_stay_unwired_until_neutral_contracts_exist() {
    for source in DEFERRED_RENDERER_TEST_SOURCES {
        let path_attribute = format!("#[path = \"test_sources/{source}.rs\"]");
        let module_declaration = format!("mod {source};");

        assert!(
            !MODULE_WIRING.contains(&path_attribute)
                && !MODULE_WIRING.contains(&module_declaration),
            "{source}.rs still carries migrated renderer-fixture assumptions; migrate it to plugin-local/public neutral seams before wiring it into the plugin test tree"
        );
    }
}
