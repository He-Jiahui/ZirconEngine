use super::*;

#[test]
fn runtime_15_asset_artifact_store_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/artifact_store.rs");
    let binary_payloads = read_runtime_src("asset/tests/assets/artifact_store/binary_payloads.rs");
    let library_assets = read_runtime_src("asset/tests/assets/artifact_store/library_assets.rs");
    let material_data = read_runtime_src("asset/tests/assets/artifact_store/material_data.rs");
    let scene_components =
        read_runtime_src("asset/tests/assets/artifact_store/scene_components.rs");
    let scene_script = read_runtime_src("asset/tests/assets/artifact_store/scene_script.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_asset_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );

    assert_contains_all(
        "asset artifact store parent test module mounts",
        &parent,
        &[
            "mod binary_payloads;",
            "mod library_assets;",
            "mod material_data;",
            "mod scene_components;",
            "mod scene_script;",
            "fn assert_binary_artifact_payload",
            "fn asset_reference",
        ],
    );
    for moved_test in [
        "fn artifact_store_roundtrips_material_assets_in_library",
        "fn artifact_store_roundtrips_material_assets_with_dynamic_property_values",
        "fn artifact_store_roundtrips_data_assets_with_dynamic_json_values",
        "fn artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
        "fn artifact_store_roundtrips_scene_assets_with_mesh_references",
        "fn artifact_store_roundtrips_scene_assets_with_camera_targets",
        "fn artifact_store_roundtrips_scene_assets_with_physics_components",
        "fn artifact_store_bincode_roundtrips_asset_reference",
        "fn artifact_store_bincode_roundtrips_scene_mesh_instance_asset",
        "fn artifact_store_roundtrips_mesh_assets_with_binary_attribute_payloads",
        "fn artifact_store_roundtrips_texture_assets_with_binary_payloads",
        "fn artifact_store_roundtrips_physics_material_assets_in_library",
        "fn artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata",
        "fn artifact_store_roundtrips_animation_sequence_assets_in_binary_library",
        "fn artifact_store_rejects_text_library_artifacts",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/artifact_store.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/artifact_store.rs should not keep executable tests in the parent module"
    );

    let migrated_child_sources = [
        material_data.as_str(),
        scene_script.as_str(),
        scene_components.as_str(),
        binary_payloads.as_str(),
        library_assets.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        15,
        "asset artifact store child modules should preserve the current 15 tests"
    );

    assert_contains_all(
        "artifact store material/data child owns material and data payload contracts",
        &material_data,
        &[
            "use super::*;",
            "fn artifact_store_roundtrips_material_assets_in_library",
            "fn artifact_store_roundtrips_material_assets_with_dynamic_property_values",
            "fn artifact_store_roundtrips_data_assets_with_dynamic_json_values",
        ],
    );
    assert_contains_all(
        "artifact store scene script child owns dynamic script binding payloads",
        &scene_script,
        &[
            "use super::*;",
            "fn artifact_store_roundtrips_scene_assets_with_script_binding_json_values",
        ],
    );
    assert_contains_all(
        "artifact store scene component child owns scene reference payloads",
        &scene_components,
        &[
            "use super::*;",
            "fn artifact_store_roundtrips_scene_assets_with_mesh_references",
            "fn artifact_store_roundtrips_scene_assets_with_camera_targets",
            "fn artifact_store_roundtrips_scene_assets_with_physics_components",
        ],
    );
    assert_contains_all(
        "artifact store binary child owns bincode and binary payload contracts",
        &binary_payloads,
        &[
            "use super::*;",
            "fn artifact_store_bincode_roundtrips_asset_reference",
            "fn artifact_store_bincode_roundtrips_scene_mesh_instance_asset",
            "fn artifact_store_roundtrips_mesh_assets_with_binary_attribute_payloads",
            "fn artifact_store_roundtrips_texture_assets_with_binary_payloads",
        ],
    );
    assert_contains_all(
        "artifact store library child owns library artifact contracts",
        &library_assets,
        &[
            "use super::*;",
            "fn artifact_store_roundtrips_physics_material_assets_in_library",
            "fn artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata",
            "fn artifact_store_roundtrips_animation_sequence_assets_in_binary_library",
            "fn artifact_store_rejects_text_library_artifacts",
        ],
    );

    for (path, source) in [
        ("asset/tests/assets/artifact_store.rs", parent.as_str()),
        (
            "asset/tests/assets/artifact_store/material_data.rs",
            material_data.as_str(),
        ),
        (
            "asset/tests/assets/artifact_store/scene_script.rs",
            scene_script.as_str(),
        ),
        (
            "asset/tests/assets/artifact_store/scene_components.rs",
            scene_components.as_str(),
        ),
        (
            "asset/tests/assets/artifact_store/binary_payloads.rs",
            binary_payloads.as_str(),
        ),
        (
            "asset/tests/assets/artifact_store/library_assets.rs",
            library_assets.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render asset doc", render_asset_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset artifact store test folder split",
                "runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred",
                "asset/tests/assets/artifact_store.rs",
                "asset/tests/assets/artifact_store/binary_payloads.rs",
                "asset/tests/assets/artifact_store/library_assets.rs",
                "runtime_15_asset_artifact_store_tests_are_folder_backed",
            ],
        );
    }
}
