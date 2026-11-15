//! 保护命名边界的子模块职责、检查入口与源码规模；这些约束读取检出文本，具体业务结果由被列出的行为测试另行验证。
use super::*;

const STATUS: &str =
    "runtime_15_ui_platform_input_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 UI platform-input guard child-owner split";
const GUARD: &str = "runtime_15_ui_platform_input_guards_are_child_owner";

#[test]
fn runtime_15_ui_platform_input_guards_are_child_owner() {
    let parent_path = "tests/contracts/naming_boundary/runtime_15_m2/ui.rs";
    let child_path = "tests/contracts/runtime_15_m2/ui/platform_input.rs";
    let parent = read_runtime_src(parent_path);
    let child = read_runtime_src(child_path);

    assert_contains_all(
        "Runtime 15 UI naming parent mounts platform-input child owner",
        &parent,
        &[
            "#[path = \"../../runtime_15_m2/ui/platform_input.rs\"]\nmod platform_input;",
            "fn runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
            "fn runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
        ],
    );
    for moved_test in [
        "fn runtime_15_platform_input_uses_dom_keycode_names",
        "fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/ui.rs should mount the platform_input child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 UI platform-input child owns platform input naming guards",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_platform_input_uses_dom_keycode_names",
            "fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
            "dom_key_code",
            "runtime_input_baseline",
        ],
    );

    // Runtime09 retired the schema-name guard; the loader regression owns the live contract.
    let asset_fixture = read_runtime_src("ui/tests/asset/fixture_migration.rs");
    assert_contains_all(
        "UI asset loader regression rejects source templates without an asset header",
        &asset_fixture,
        &[
            "fn ui_asset_loader_rejects_source_template_documents_without_asset_header",
            "UiAssetLoader::load_toml_str(SOURCE_TEMPLATE_WITHOUT_ASSET_HEADER_TOML)",
            "ui asset source is missing [asset]",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_ui_template_schema_uses_source_fixture_names"),
        "the retired UI template schema naming guard should not return to the naming parent"
    );

    for (path, source) in [(parent_path, parent.as_str()), (child_path, child.as_str())] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let _runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let _runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let _structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
}
