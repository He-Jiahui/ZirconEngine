//! 核对资源制品存储测试迁移后的父子分工与数量快照，并检查文件预算；出现次数来自源码文本，后续新增测试需按当前职责复核快照。
use super::*;

#[test]
fn runtime_15_asset_artifact_store_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/artifact_store.rs");
    let binary_payloads = read_runtime_src("asset/tests/assets/artifact_store/binary_payloads.rs");
    let artifact_cache_assets =
        read_runtime_src("asset/tests/assets/artifact_store/artifact_cache_assets.rs");
    let material_data = read_runtime_src("asset/tests/assets/artifact_store/material_data.rs");
    let scene_components =
        read_runtime_src("asset/tests/assets/artifact_store/scene_components.rs");
    let scene_script = read_runtime_src("asset/tests/assets/artifact_store/scene_script.rs");
    let bounded_read = read_runtime_src("asset/tests/assets/artifact_store/bounded_read.rs");
    let lazy_residency = read_runtime_src("asset/tests/assets/artifact_store/lazy_residency.rs");
    let prefab_overrides =
        read_runtime_src("asset/tests/assets/artifact_store/prefab_overrides.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _render_asset_doc = read_repo("docs/crates/zircon_runtime/asset/render-assets.md");

    assert_contains_all(
        "asset artifact store parent test module mounts",
        &parent,
        &[
            "#[path = \"artifact_store/binary_payloads.rs\"]\nmod binary_payloads;",
            "#[path = \"artifact_store/artifact_cache_assets.rs\"]\nmod artifact_cache_assets;",
            "#[path = \"artifact_store/material_data.rs\"]\nmod material_data;",
            "#[path = \"artifact_store/scene_components.rs\"]\nmod scene_components;",
            "#[path = \"artifact_store/scene_script.rs\"]\nmod scene_script;",
            "#[path = \"artifact_store/bounded_read.rs\"]\nmod bounded_read;",
            "#[path = \"artifact_store/lazy_residency.rs\"]\nmod lazy_residency;",
            "#[path = \"artifact_store/prefab_overrides.rs\"]\nmod prefab_overrides;",
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
        "fn artifact_store_roundtrips_physics_material_assets_in_artifact_cache",
        "fn artifact_store_regenerates_shader_material_artifact_from_cache_schema",
        "fn artifact_store_roundtrips_animation_sequence_assets_in_binary_artifact_cache",
        "fn artifact_store_rejects_text_artifact_cache_artifacts",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/artifact_store.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        16,
        "artifact store parent should retain its 16 stream, manifest and publication guards; migrated payload tests stay in their children"
    );

    let migrated_child_sources = [
        material_data.as_str(),
        scene_script.as_str(),
        scene_components.as_str(),
        binary_payloads.as_str(),
        artifact_cache_assets.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        21,
        "artifact store migrated children should retain 3 material/data, 1 script, 6 scene, 6 binary and 5 cache tests"
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
        "artifact store cache child owns artifact-cache contracts",
        &artifact_cache_assets,
        &[
            "use super::*;",
            "fn artifact_store_roundtrips_physics_material_assets_in_artifact_cache",
            "fn artifact_store_regenerates_shader_material_artifact_from_cache_schema",
            "fn artifact_store_roundtrips_animation_sequence_assets_in_binary_artifact_cache",
            "fn artifact_store_rejects_text_artifact_cache_artifacts",
        ],
    );

    assert_contains_all(
        "artifact store prefab child owns independent and nested override roundtrips",
        &prefab_overrides,
        &[
            "fn unreal_equiv_artifact_store_roundtrips_scene_prefab_instance_overrides",
            "fn unreal_equiv_artifact_store_roundtrips_prefab_source_with_nested_instance_overrides",
        ],
    );
    assert_contains_all(
        "artifact store bounded-read child owns rejection before chunk I/O",
        &bounded_read,
        &["fn artifact_store_rejects_raw_payload_over_read_budget_before_opening_chunks"],
    );
    assert_contains_all(
        "artifact store residency child owns consumer leases and last-good publication",
        &lazy_residency,
        &[
            "fn artifact_store_lazily_resides_only_requested_compressed_chunks",
            "fn artifact_store_trim_releases_cache_owned_payloads_but_preserves_consumer_leases",
            "fn artifact_store_unpublished_prepared_generation_keeps_last_good_manifest",
        ],
    );

    for (path, source, expected_tests) in [
        ("asset/tests/assets/artifact_store.rs", parent.as_str(), 16),
        (
            "asset/tests/assets/artifact_store/material_data.rs",
            material_data.as_str(),
            3,
        ),
        (
            "asset/tests/assets/artifact_store/scene_script.rs",
            scene_script.as_str(),
            1,
        ),
        (
            "asset/tests/assets/artifact_store/scene_components.rs",
            scene_components.as_str(),
            6,
        ),
        (
            "asset/tests/assets/artifact_store/binary_payloads.rs",
            binary_payloads.as_str(),
            6,
        ),
        (
            "asset/tests/assets/artifact_store/artifact_cache_assets.rs",
            artifact_cache_assets.as_str(),
            5,
        ),
        (
            "asset/tests/assets/artifact_store/bounded_read.rs",
            bounded_read.as_str(),
            1,
        ),
        (
            "asset/tests/assets/artifact_store/lazy_residency.rs",
            lazy_residency.as_str(),
            10,
        ),
        (
            "asset/tests/assets/artifact_store/prefab_overrides.rs",
            prefab_overrides.as_str(),
            2,
        ),
    ] {
        assert_eq!(
            source.matches("#[test]").count(),
            expected_tests,
            "{path} should preserve its current owned tests"
        );
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
