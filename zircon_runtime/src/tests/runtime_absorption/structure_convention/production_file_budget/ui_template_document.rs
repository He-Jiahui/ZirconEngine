use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_template_document_validation_is_child_owner() {
    let parent = read_runtime_src("ui/template/asset/document.rs");
    let validation = read_runtime_src("ui/template/asset/document/validation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "UI asset document parent keeps runtime extension API and tree mutation helpers",
        &parent,
        &[
            "mod validation;",
            "use validation::{",
            "pub trait UiAssetDocumentRuntimeExt",
            "impl UiAssetDocumentRuntimeExt for UiAssetDocument",
            "pub struct UiAssetNodeIter",
            "pub struct UiNodeParent",
            "pub struct UiStyleRulePosition",
            "fn find_node<",
            "fn replace_node_in_tree(",
            "fn remove_node_from_tree(",
        ],
    );
    for moved_owner in [
        "fn validate_node_tree(",
        "fn validate_stylesheet_ids(",
        "fn validate_style_rule_ids(",
        "fn validate_style_rule_selectors(",
        "UiSelector::parse",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/template/asset/document.rs should delegate validation owner `{moved_owner}` to validation.rs"
        );
    }
    assert_contains_all(
        "validation child owns document identity and selector checks",
        &validation,
        &[
            "pub(super) fn validate_node_tree",
            "pub(super) fn validate_stylesheet_ids",
            "pub(super) fn validate_style_rule_ids",
            "pub(super) fn validate_style_rule_selectors",
            "UiSelector::parse",
            "duplicate node_id",
            "duplicate stylesheet id",
            "duplicate style rule id",
        ],
    );

    for (path, source) in [
        ("ui/template/asset/document.rs", parent.as_str()),
        (
            "ui/template/asset/document/validation.rs",
            validation.as_str(),
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
                "Runtime 15 M4 UI template document validation owner split",
                "runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred",
                "ui/template/asset/document.rs",
                "ui/template/asset/document/validation.rs",
                "runtime_15_ui_template_document_validation_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI template document validation owner split",
            "runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred",
            "ui/template/asset/document.rs",
            "ui/template/asset/document/validation.rs",
            "runtime_15_ui_template_document_validation_is_child_owner",
        ],
    );
}
