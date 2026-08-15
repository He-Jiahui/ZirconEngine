use super::*;

#[test]
fn runtime_15_p0_native_fixture_review_guards_are_leaf_owners() {
    let parent = read_runtime_src(PARENT);
    let sdk_macro_leaf = read_runtime_src(SDK_MACRO_LEAF);
    let importer_leaf = read_runtime_src(IMPORTER_LEAF);

    assert_contains_all(
        "P0 native fixture parent only mounts leaf review guard owners",
        &parent,
        &[
            "#[path = \"native_fixture/importer_manifest.rs\"]",
            "mod importer_manifest;",
            "#[path = \"native_fixture/sdk_macro_manifest.rs\"]",
            "mod sdk_macro_manifest;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "{PARENT} should stay a route-only parent"
    );
    assert_contains_all(
        "SDK macro leaf owns D-S8/D3 native fixture review guard",
        &sdk_macro_leaf,
        &[
            "fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "zircon_plugin_sdk::native_dist_plugin_v3!",
            "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "importer manifest leaf owns D13 native fixture review guard",
        &importer_leaf,
        &[
            "fn review_d13_native_fixture_importer_is_manifest_described",
            "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
            "runtime.asset.importer.native_dynamic_fixture.data_json",
        ],
    );
}
