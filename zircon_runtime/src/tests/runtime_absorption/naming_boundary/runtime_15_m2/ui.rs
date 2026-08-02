use std::path::Path;

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows, read_text,
};

#[path = "ui/platform_input.rs"]
mod platform_input;

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
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let ui_doc = read_repo_text(manifest_root, "docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

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
        &manifest_root.join("src/tests/runtime_absorption/naming_boundary/classifiers.rs"),
        "runtime naming boundary guard should be readable",
    );
    let python_audit = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let ui_doc = read_repo_text(manifest_root, "docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

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
