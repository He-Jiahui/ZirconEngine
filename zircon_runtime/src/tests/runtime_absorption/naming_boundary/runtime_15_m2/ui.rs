use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let editor_showcase_dir = manifest_root.join("src/ui/component/catalog/editor_showcase");
    let retired_helpers = editor_showcase_dir.join("helpers.rs");
    let parent = read_text(
        &manifest_root.join("src/ui/component/catalog/editor_showcase.rs"),
        "editor showcase parent should be readable",
    );
    let descriptor_builders = read_text(
        &editor_showcase_dir.join("descriptor_builders.rs"),
        "editor showcase descriptor builders owner should be readable",
    );
    let production_guard = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/structure_convention/production_file_budget/ui_component_catalog_editor_showcase.rs",
        ),
        "editor showcase production-budget guard should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let ui_doc = read_repo_text(manifest_root, "docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_helpers.exists(),
        "editor showcase should not keep banned-name helper owner {:?}",
        retired_helpers
    );
    assert_contains_all(
        "editor showcase parent",
        &parent,
        &["mod descriptor_builders;", "use descriptor_builders::{"],
    );
    assert!(
        !parent.contains("mod helpers;") && !parent.contains("use helpers::"),
        "editor_showcase.rs should not preserve the retired helpers module name"
    );
    assert_contains_all(
        "editor showcase descriptor builders owner",
        &descriptor_builders,
        &[
            "fn base_descriptor(",
            "pub(super) fn with_palette_metadata",
            "pub(super) fn layout_primitive",
            "pub(super) fn options_prop()",
            "UiWidgetFallbackPolicy",
            "UiPaletteMetadata",
        ],
    );
    assert_contains_all(
        "editor showcase production-budget guard",
        &production_guard,
        &["ui/component/catalog/editor_showcase/descriptor_builders.rs"],
    );
    assert!(
        !production_guard.contains("ui/component/catalog/editor_showcase/helpers.rs"),
        "production-budget guard should not keep retired editor showcase helper path"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("UI architecture doc", ui_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover",
                "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred",
                "ui/component/catalog/editor_showcase/descriptor_builders.rs",
                "runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let columns = read_text(
        &manifest_root.join("src/ui/surface/surface/default_interactions/table/columns.rs"),
        "UI table column helper owner should be readable",
    );
    let rust_guard = read_text(
        &manifest_root.join("src/tests/runtime_absorption/naming_boundary.rs"),
        "runtime naming boundary guard should be readable",
    );
    let python_audit = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let ui_doc = read_repo_text(manifest_root, "docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert_contains_all(
        "UI table column helper owner",
        &columns,
        &[
            "pub(super) fn table_uses_client_sorting",
            "string_attribute(metadata, \"sortingMode\")",
            "Some(\"server\")",
        ],
    );
    assert_contains_all(
        "Rust non-network server naming guard",
        &rust_guard,
        &[
            "src/ui/surface/surface/default_interactions/table/columns.rs",
            "line.contains(\"Some(\\\"server\\\")\")",
        ],
    );
    assert_contains_all(
        "Python non-network server naming audit",
        &python_audit,
        &[
            "zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs",
            "Some(\"server\")",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("UI architecture doc", ui_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 UI table sortingMode server literal allowed-context sync",
                "runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred",
                "ui/surface/surface/default_interactions/table/columns.rs",
                "runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
            ],
        );
    }
}

#[test]
fn runtime_15_platform_input_uses_dom_keycode_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let keyboard_map = read_text(
        &manifest_root.join("src/ui/platform_input/keyboard_map.rs"),
        "platform input keyboard map should be readable",
    );
    let winit_translation = read_text(
        &manifest_root.join("src/ui/platform_input/winit_translation.rs"),
        "platform input winit translation should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let platform_input_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/ui/platform_input.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected date slice should be readable",
    );

    assert_contains_all(
        "platform input keyboard map",
        &keyboard_map,
        &[
            "pub(super) fn dom_key_code",
            "Key::Character(text) => dom_character_key_code(text)",
            "fn dom_character_key_code",
        ],
    );
    assert!(
        !keyboard_map.contains("legacy_key_code")
            && !keyboard_map.contains("legacy_character_key_code"),
        "platform input keyboard map should not keep legacy key-code helper names"
    );
    assert_contains_all(
        "platform input winit translation",
        &winit_translation,
        &[
            "dom_key_code(&event.logical_key)",
            "const PIXEL_SCROLL_LINE_DELTA_SCALE",
            "translate_winit_wheel_preserves_precise_delta_and_line_delta_scale",
        ],
    );
    assert!(
        !winit_translation.contains("legacy_key_code")
            && !winit_translation.contains("PIXEL_SCROLL_LEGACY_LINE_SCALE")
            && !winit_translation.contains("legacy_scalar"),
        "platform input winit translation should not keep legacy naming"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("platform input doc", platform_input_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 platform input DOM keycode naming hard cutover",
                "runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "ui/platform_input/keyboard_map.rs",
                "dom_key_code",
                "runtime_15_platform_input_uses_dom_keycode_names",
            ],
        );
    }
}

#[test]
fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let winit_translation = read_text(
        &manifest_root.join("src/ui/platform_input/winit_translation.rs"),
        "platform input winit translation should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let platform_input_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/ui/platform_input.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 foundation status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected date slice should be readable",
    );

    assert_contains_all(
        "platform input winit translation runtime-input baseline tests",
        &winit_translation,
        &[
            "translate_winit_keyboard_matrix_matches_runtime_input_baseline",
            "translate_winit_ime_preedit_commit_and_disable_match_runtime_input_baseline",
        ],
    );
    assert!(
        !winit_translation.contains("editor_baseline"),
        "platform input runtime tests should not use editor_baseline names inside the runtime owner"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("platform input doc", platform_input_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 platform input runtime baseline test naming hard cutover",
                "runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred",
                "ui/platform_input/winit_translation.rs",
                "runtime_input_baseline",
                "runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
            ],
        );
    }
}

#[test]
fn runtime_15_ui_template_schema_uses_source_fixture_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let schema_report = read_repo_text(
        manifest_root,
        "zircon_runtime_interface/src/ui/template/asset/schema/report.rs",
    );
    let schema_migrator = read_text(
        &manifest_root.join("src/ui/template/asset/schema/migrator.rs"),
        "UI template schema migrator should be readable",
    );
    let schema_tests = read_text(
        &manifest_root.join("src/ui/tests/asset_schema_migration.rs"),
        "UI asset schema migration tests should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let template_doc = read_repo_text(manifest_root, "docs/zircon_runtime/ui/template/pipeline.md");
    let interface_doc = read_repo_text(manifest_root, "docs/zircon_runtime_interface/ui/mod.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary expected date slice should be readable",
    );

    assert_contains_all(
        "UI asset schema report interface",
        &schema_report,
        &[
            "UiAssetSchemaSourceKind::FutureVersion",
            "SourceTemplateFixture",
            "SourceTemplateFixtureConverted",
        ],
    );
    assert!(
        !schema_report.contains("LegacyTemplateFixture")
            && !schema_report.contains("LegacyTemplateConverted"),
        "UI asset schema report should not expose legacy template fixture enum names"
    );
    assert_contains_all(
        "UI template schema migrator",
        &schema_migrator,
        &[
            "UiAssetSchemaSourceKind::SourceTemplateFixture",
            "UiAssetMigrationStep::SourceTemplateFixtureConverted",
        ],
    );
    assert!(
        !schema_migrator.contains("LegacyTemplateFixture")
            && !schema_migrator.contains("LegacyTemplateConverted"),
        "UI template schema migrator should not call retired legacy template enum variants"
    );
    assert_contains_all(
        "UI asset schema migration tests",
        &schema_tests,
        &[
            "SOURCE_TEMPLATE_FIXTURE_TOML",
            "SourceFixtureRoot",
            "UiAssetSchemaSourceKind::SourceTemplateFixture",
            "UiAssetMigrationStep::SourceTemplateFixtureConverted",
        ],
    );
    assert!(
        !schema_tests.contains("LEGACY_TEMPLATE_TOML"),
        "UI asset schema migration tests should use source fixture vocabulary"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("UI template pipeline doc", template_doc),
        ("runtime interface UI doc", interface_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 UI template schema source fixture naming hard cutover",
                "runtime_15_ui_template_schema_source_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "ui/template/asset/schema/migrator.rs",
                "SourceTemplateFixture",
                "runtime_15_ui_template_schema_uses_source_fixture_names",
            ],
        );
    }
}
