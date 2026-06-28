use super::*;

#[test]
fn runtime_15_asset_project_flow_sample_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/project/asset_flow_sample.rs");
    let end_to_end = read_runtime_src("asset/tests/project/asset_flow_sample/end_to_end.rs");
    let importers = read_runtime_src("asset/tests/project/asset_flow_sample/importers.rs");
    let fixtures = read_runtime_src("asset/tests/project/asset_flow_sample/fixtures.rs");
    let assertions = read_runtime_src("asset/tests/project/asset_flow_sample/assertions.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "asset project flow sample parent test module mounts",
        &parent,
        &[
            "mod assertions;",
            "mod end_to_end;",
            "mod fixtures;",
            "mod importers;",
        ],
    );
    for moved_item in [
        "fn project_manager_imports_minimal_gltf_material_shader_mesh_sample",
        "fn project_manager_with_sample_importers",
        "fn write_minimal_textured_gltf",
        "fn assert_ready_record",
    ] {
        assert!(
            !parent.contains(moved_item),
            "asset/tests/project/asset_flow_sample.rs should mount child owners instead of defining {moved_item}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/project/asset_flow_sample.rs should not keep executable tests in the parent module"
    );
    assert_eq!(
        end_to_end.matches("#[test]").count(),
        1,
        "asset flow end-to-end child should preserve the original sample test"
    );

    assert_contains_all(
        "asset flow end-to-end child owns the sample assertion path",
        &end_to_end,
        &[
            "use super::assertions::{",
            "fn project_manager_imports_minimal_gltf_material_shader_mesh_sample",
            "ResourceStreamer::new_for_test",
        ],
    );
    assert_contains_all(
        "asset flow importer child owns sample importer wiring",
        &importers,
        &[
            "pub(super) fn project_manager_with_sample_importers",
            "pub(super) fn project_asset_manager_with_sample_importers",
            "fn dds_container_importer",
        ],
    );
    assert_contains_all(
        "asset flow fixture child owns sample source writers",
        &fixtures,
        &[
            "pub(super) fn write_minimal_textured_gltf",
            "pub(super) fn write_sample_shader_package",
            "pub(super) fn write_bc1_texture",
            "fn dds_classic_fourcc_bytes",
        ],
    );
    assert_contains_all(
        "asset flow assertion child owns loading and dependency helpers",
        &assertions,
        &[
            "pub(super) fn assert_ready_record",
            "pub(super) fn assert_dependencies",
            "pub(super) fn texture_bind_group_layout",
            "pub(super) fn uri",
        ],
    );

    for source in [
        parent.as_str(),
        end_to_end.as_str(),
        importers.as_str(),
        fixtures.as_str(),
        assertions.as_str(),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset project flow sample parent and child owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset project flow sample test folder split",
                "runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked",
                "asset/tests/project/asset_flow_sample.rs",
                "asset/tests/project/asset_flow_sample/end_to_end.rs",
                "runtime_15_asset_project_flow_sample_tests_are_folder_backed",
            ],
        );
    }
}
