use super::*;

const STATUS: &str =
    "runtime_15_asset_schema_material_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 asset-schema material guard child-owner split";
const GUARD: &str = "runtime_15_asset_schema_material_guard_is_child_owner";

#[test]
fn runtime_15_asset_schema_material_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema/material_asset_schema_v1.rs",
    );

    assert_contains_all(
        "Runtime 15 asset-schema parent mounts material child owner",
        &parent,
        &[
            "#[path = \"asset_schema/material_asset_schema_v1.rs\"]",
            "mod material_asset_schema_v1;",
            "fn runtime_15_font_ui_asset_schema_names_use_current_policy_terms",
            "fn runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_material_asset_schema_v1_defaults_use_versioned_names"),
        "runtime_15_m2/asset_schema.rs should mount material_asset_schema_v1 child instead of defining the naming guard"
    );

    assert_contains_all(
        "Runtime 15 asset-schema material child owns guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_material_asset_schema_v1_defaults_use_versioned_names",
            "asset/assets/material/material_asset.rs",
            "property_overrides_with_schema_v1_defaults",
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema/material_asset_schema_v1.rs",
            child.as_str(),
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
}
