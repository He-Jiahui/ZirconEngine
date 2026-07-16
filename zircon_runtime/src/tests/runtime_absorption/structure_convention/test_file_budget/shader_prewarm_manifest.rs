use super::*;

#[path = "shader_prewarm_manifest/asset_revision.rs"]
mod asset_revision;
#[path = "shader_prewarm_manifest/builtin_fallback.rs"]
mod builtin_fallback;
#[path = "shader_prewarm_manifest/builtin_template_source.rs"]
mod builtin_template_source;
#[path = "shader_prewarm_manifest/custom_shading_model.rs"]
mod custom_shading_model;
#[path = "shader_prewarm_manifest/geometry_source.rs"]
mod geometry_source;
#[path = "shader_prewarm_manifest/manifest_contract.rs"]
mod manifest_contract;
#[path = "shader_prewarm_manifest/product_staged_prewarm.rs"]
mod product_staged_prewarm;

const PARENT_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs";
const ASSET_REVISION_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/asset_revision.rs";
const BUILTIN_FALLBACK_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/builtin_fallback.rs";
const BUILTIN_TEMPLATE_SOURCE_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/builtin_template_source.rs";
const CUSTOM_SHADING_MODEL_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/custom_shading_model.rs";
const GEOMETRY_SOURCE_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs";
const MANIFEST_CONTRACT_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/manifest_contract.rs";
const PRODUCT_STAGED_PREWARM_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/product_staged_prewarm.rs";

#[test]
fn runtime_15_shader_prewarm_manifest_guard_children_are_folder_backed() {
    let parent = read_runtime_src(PARENT_OWNER);
    let asset_revision = read_runtime_src(ASSET_REVISION_OWNER);
    let builtin_fallback = read_runtime_src(BUILTIN_FALLBACK_OWNER);
    let builtin_template_source = read_runtime_src(BUILTIN_TEMPLATE_SOURCE_OWNER);
    let custom_shading_model = read_runtime_src(CUSTOM_SHADING_MODEL_OWNER);
    let geometry_source = read_runtime_src(GEOMETRY_SOURCE_OWNER);
    let manifest_contract = read_runtime_src(MANIFEST_CONTRACT_OWNER);
    let product_staged_prewarm = read_runtime_src(PRODUCT_STAGED_PREWARM_OWNER);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    ) + &read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    assert_contains_all(
        "shader prewarm manifest guard parent mounts child owners",
        &parent,
        &[
            "mod asset_revision;",
            "mod builtin_fallback;",
            "mod builtin_template_source;",
            "mod custom_shading_model;",
            "mod geometry_source;",
            "mod manifest_contract;",
            "mod product_staged_prewarm;",
            "fn runtime_15_shader_prewarm_manifest_guard_children_are_folder_backed",
        ],
    );
    for moved_anchor in [
        concat!(
            "fn runtime_15_shader_prewarm_manifest_tests_are_",
            "folder_backed"
        ),
        concat!(
            "fn runtime_15_shader_prewarm_geometry_source_",
            "enumeration_is_wired"
        ),
        concat!(
            "fn runtime_15_shader_prewarm_custom_geometry_source_",
            "id_is_wired"
        ),
        concat!(
            "fn runtime_15_shader_prewarm_custom_shading_model_",
            "id_is_wired"
        ),
        concat!(
            "fn runtime_15_shader_prewarm_builtin_standard_material_",
            "template_source_is_wired"
        ),
        concat!(
            "fn runtime_15_builtin_fallback_prewarm_uses_",
            "template_source"
        ),
        concat!(
            "fn runtime_15_product_base_mesh_staged_",
            "prewarm_is_wired"
        ),
        concat!(
            "fn runtime_15_shader_prewarm_asset_revision_",
            "export_is_wired"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "shader_prewarm_manifest.rs should mount guard children instead of retaining `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "manifest contract child keeps original folder-backed guard",
        &manifest_contract,
        &[concat!(
            "fn runtime_15_shader_prewarm_manifest_tests_are_",
            "folder_backed"
        )],
    );
    assert_contains_all(
        "geometry source child keeps Plan 08 geometry-source guard",
        &geometry_source,
        &[
            concat!(
                "fn runtime_15_shader_prewarm_geometry_source_",
                "enumeration_is_wired"
            ),
            concat!(
                "fn runtime_15_shader_prewarm_custom_geometry_source_",
                "id_is_wired"
            ),
        ],
    );
    assert_contains_all(
        "custom shading model child keeps explicit id guard",
        &custom_shading_model,
        &[concat!(
            "fn runtime_15_shader_prewarm_custom_shading_model_",
            "id_is_wired"
        )],
    );
    assert_contains_all(
        "builtin template child keeps standard material template guard",
        &builtin_template_source,
        &[concat!(
            "fn runtime_15_shader_prewarm_builtin_standard_material_",
            "template_source_is_wired"
        )],
    );
    assert_contains_all(
        "builtin fallback child keeps template source alignment guard",
        &builtin_fallback,
        &[concat!(
            "fn runtime_15_builtin_fallback_prewarm_uses_",
            "template_source"
        )],
    );
    assert_contains_all(
        "product staged prewarm child keeps product guard",
        &product_staged_prewarm,
        &[concat!(
            "fn runtime_15_product_base_mesh_staged_",
            "prewarm_is_wired"
        )],
    );
    assert_contains_all(
        "asset revision child keeps revision export guard",
        &asset_revision,
        &[concat!(
            "fn runtime_15_shader_prewarm_asset_revision_",
            "export_is_wired"
        )],
    );

    let slice = "Runtime 15 M3 shader prewarm manifest guard child-owner split";
    let status =
        "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred";
    let guard = "runtime_15_shader_prewarm_manifest_guard_children_are_folder_backed";
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md")
        + &read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
        );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    for (label, source) in [
        ("status rows", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                slice,
                status,
                "structure_convention/test_file_budget/shader_prewarm_manifest.rs",
                "structure_convention/test_file_budget/shader_prewarm_manifest/manifest_contract.rs",
                "structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs",
                "structure_convention/test_file_budget/shader_prewarm_manifest/builtin_template_source.rs",
                "structure_convention/test_file_budget/shader_prewarm_manifest/asset_revision.rs",
                guard,
            ],
        );
    }
    assert_contains_all(
        "status map records shader prewarm manifest guard split status",
        &status_map,
        &[slice, status],
    );
    assert_contains_all(
        "date map records shader prewarm manifest guard split date",
        &date_map,
        &[slice, "Some(\"2026-06-27\")"],
    );

    for owner in [
        PARENT_OWNER,
        ASSET_REVISION_OWNER,
        BUILTIN_FALLBACK_OWNER,
        BUILTIN_TEMPLATE_SOURCE_OWNER,
        CUSTOM_SHADING_MODEL_OWNER,
        GEOMETRY_SOURCE_OWNER,
        MANIFEST_CONTRACT_OWNER,
        PRODUCT_STAGED_PREWARM_OWNER,
    ] {
        let source = read_runtime_src(owner);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{owner} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
