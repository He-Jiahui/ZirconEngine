const MODULE_WIRING: &str = include_str!("../mod.rs");

const DEFERRED_RENDERER_TEST_SOURCES: &[&str] = &[
    "hybrid_gi_gpu",
    "hybrid_gi_gpu_hierarchy",
    "hybrid_gi_gpu_runtime_source",
    "hybrid_gi_gpu_scene_light_seed",
    "hybrid_gi_resolve_dynamic_lights",
    "hybrid_gi_resolve_history",
    "hybrid_gi_resolve_render",
    "hybrid_gi_resolve_surface_cache",
    "hybrid_gi_scene_prepare_resources",
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
