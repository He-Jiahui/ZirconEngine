use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner() {
    let parent = read_runtime_src("ui/component/catalog/editor_showcase.rs");
    let descriptors = read_runtime_src("ui/component/catalog/editor_showcase/descriptors.rs");
    let descriptor_builders =
        read_runtime_src("ui/component/catalog/editor_showcase/descriptor_builders.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "editor showcase parent owns only catalog registration and execution",
        &parent,
        &[
            "mod descriptor_builders;",
            "mod descriptors;",
            "use self::descriptors::editor_showcase_descriptors;",
            "static EDITOR_SHOWCASE_REGISTRY",
            "fn build_editor_showcase_registry",
        ],
    );
    for declaration in [
        "fn build_editor_showcase_descriptor",
        "fn editor_showcase_descriptors()",
        "layout_primitive(\"Container\"",
        "selection(\"Dropdown\"",
        "editor_collection(\"TreeView\"",
        "\"context-action-menu\"",
    ] {
        assert!(
            !parent.contains(declaration),
            "ui/component/catalog/editor_showcase.rs must delegate catalog declaration `{declaration}` to editor_showcase/descriptors.rs"
        );
    }
    assert_contains_all(
        "editor showcase descriptor child owns generated catalog declarations",
        &descriptors,
        &[
            "use super::descriptor_builders::{",
            "fn build_editor_showcase_descriptor",
            "pub(super) fn editor_showcase_descriptors()",
            "layout_primitive(\"Container\"",
            "selection(\"Dropdown\"",
            "editor_collection(\"TreeView\"",
            "\"ContextActionMenu\"",
            "\"context-action-menu\"",
        ],
    );
    for moved_owner in [
        "fn base_descriptor(",
        "fn layout_role_for(",
        "fn default_template_from_descriptor(",
        "fn category_sort_key(",
        "fn options_prop()",
        "fn expanded_prop()",
        "UiWidgetFallbackPolicy",
        "UiPaletteMetadata",
    ] {
        assert!(
            !parent.contains(moved_owner) && !descriptors.contains(moved_owner),
            "editor showcase catalog owners should delegate descriptor builder `{moved_owner}` to editor_showcase/descriptor_builders.rs"
        );
    }
    assert_contains_all(
        "editor showcase descriptor-builders child owns descriptor construction",
        &descriptor_builders,
        &[
            "fn base_descriptor(",
            "pub(super) fn with_palette_metadata",
            "pub(super) fn layout_primitive",
            "fn layout_role_for(",
            "fn default_template_from_descriptor(",
            "fn category_sort_key(",
            "pub(super) fn options_prop()",
            "pub(super) fn expanded_prop()",
            "UiWidgetFallbackPolicy",
            "UiPaletteMetadata",
            "BTreeMap",
            "toml::Value",
        ],
    );

    for (path, source) in [
        ("ui/component/catalog/editor_showcase.rs", parent.as_str()),
        (
            "ui/component/catalog/editor_showcase/descriptors.rs",
            descriptors.as_str(),
        ),
        (
            "ui/component/catalog/editor_showcase/descriptor_builders.rs",
            descriptor_builders.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 UI component catalog editor-showcase helper owner split",
                "runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result",
                "ui/component/catalog/editor_showcase.rs",
                "ui/component/catalog/editor_showcase/descriptor_builders.rs",
                "runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner",
            ],
        );
    }

    assert!(
        ui_doc.contains("ui/component/catalog/editor_showcase/descriptors.rs"),
        "UI architecture doc should name the generated catalog declaration owner"
    );
}
