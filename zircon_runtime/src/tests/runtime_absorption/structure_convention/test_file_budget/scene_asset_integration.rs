use super::*;

#[test]
fn runtime_15_scene_asset_integration_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/asset_scene.rs");
    let mesh_bindings = read_runtime_src("scene/tests/asset_scene/mesh_bindings.rs");
    let hierarchy_sources = read_runtime_src("scene/tests/asset_scene/hierarchy_sources.rs");
    let product_fields = read_runtime_src("scene/tests/asset_scene/product_fields.rs");

    assert_contains_all(
        "scene asset integration parent keeps shared imports/helpers and mounts children",
        &parent,
        &[
            "mod hierarchy_sources;",
            "mod mesh_bindings;",
            "mod product_fields;",
            "fn asset_reference(",
            "fn project_io_source(",
            "fn project_io_section",
            "fn assert_scene_asset_excludes_authoring_tokens",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/asset_scene.rs should only keep shared helpers and mount child owners"
    );
    for moved_test in [
        "fn scene_assets_instantiate_world_with_asset_bound_meshes",
        "fn render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay",
        "fn scene_assets_roundtrip_primitive_mesh_material_bindings",
        "fn scene_assets_keep_script_only_entities_as_empty_nodes",
        "fn scene_asset_load_uses_asset_preserving_normalizer_source_guard",
        "fn scene_assets_keep_transform_only_hierarchy_nodes",
        "fn scene_assets_roundtrip_asset_bound_physics_and_animation_components",
        "fn scene_assets_roundtrip_camera_product_fields",
        "fn scene_assets_roundtrip_ambient_and_rect_light_product_fields",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved scene asset integration test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "mesh bindings child owns asset-bound mesh and primitive binding coverage",
        &mesh_bindings,
        &[
            "fn scene_assets_instantiate_world_with_asset_bound_meshes",
            "fn render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay",
            "fn scene_assets_roundtrip_primitive_mesh_material_bindings",
        ],
    );
    assert_contains_all(
        "hierarchy/source child owns script-only, source guard, and hierarchy coverage",
        &hierarchy_sources,
        &[
            "fn scene_assets_keep_script_only_entities_as_empty_nodes",
            "fn scene_asset_load_uses_asset_preserving_normalizer_source_guard",
            "fn scene_assets_keep_transform_only_hierarchy_nodes",
        ],
    );
    assert_contains_all(
        "product fields child owns physics/animation, camera, and light product coverage",
        &product_fields,
        &[
            "fn scene_assets_roundtrip_asset_bound_physics_and_animation_components",
            "fn scene_assets_roundtrip_camera_product_fields",
            "fn scene_assets_roundtrip_ambient_and_rect_light_product_fields",
        ],
    );
    let child_test_total = [
        mesh_bindings.as_str(),
        hierarchy_sources.as_str(),
        product_fields.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 9,
        "scene asset integration children should preserve all 9 parent tests"
    );

    for (path, source) in [
        ("scene/tests/asset_scene.rs", parent.as_str()),
        (
            "scene/tests/asset_scene/mesh_bindings.rs",
            mesh_bindings.as_str(),
        ),
        (
            "scene/tests/asset_scene/hierarchy_sources.rs",
            hierarchy_sources.as_str(),
        ),
        (
            "scene/tests/asset_scene/product_fields.rs",
            product_fields.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_extract_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
    let inspection_doc = read_repo("docs/zircon_runtime/scene/inspection.md");
    let render_assets_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene render extract doc", render_extract_doc.as_str()),
        ("scene inspection doc", inspection_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene asset integration test folder split",
                "runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/asset_scene.rs",
                "scene/tests/asset_scene/mesh_bindings.rs",
                "scene/tests/asset_scene/product_fields.rs",
                "runtime_15_scene_asset_integration_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M3 scene asset integration test folder split",
            "runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M3 scene asset integration test folder split",
            "2026-06-24",
        ],
    );
}
