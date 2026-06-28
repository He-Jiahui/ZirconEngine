#[test]
fn review_f5_ui_template_resource_resolver_uses_typed_lookup_errors_before_diagnostics_boundary() {
    let resolver = include_str!("../../../../ui/template/asset/resource_ref/resolver.rs");
    let resolver_tests = include_str!("../../../../ui/tests/asset_resource_resolver.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let resolver_doc = include_str!(
        "../../../../../../docs/zircon_runtime/ui/template/asset/resource_ref/resolver.md"
    );
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "ResourceLocatorError",
        "type UiResourceLookupResult<T>",
        "enum UiResourceLookupError",
        "ResourceLocator(#[from] ResourceLocatorError)",
        ") -> UiResourceLookupResult<RuntimeResourceLookup>",
        ") -> UiResourceLookupResult<(String, Option<String>)>",
        "ResourceLocator::parse(",
        "ResourceLocator::new(",
        "ResourceLocatorError::EmptyLabel.into()",
    ] {
        assert!(
            resolver.contains(required),
            "UI resource resolver typed lookup path should contain `{required}`"
        );
    }

    for forbidden in [
        "Result<RuntimeResourceLookup, String>",
        "Result<(String, Option<String>), String>",
        ".map_err(|error| error.to_string())",
        "Err(\"resource locator label cannot be empty\".to_string())",
    ] {
        assert!(
            !resolver.contains(forbidden),
            "UI resource resolver should not keep String-error transport `{forbidden}`"
        );
    }

    for required in [
        "ui_resource_resolver_reports_invalid_mapped_ui_scheme_empty_label",
        "UiResourceResolveDiagnosticCode::InvalidUri",
        "\"resource uri is invalid: resource locator label cannot be empty\"",
    ] {
        assert!(
            resolver_tests.contains(required),
            "UI resource resolver behavior tests should contain `{required}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 UI template resource resolver typed errors",
        "runtime_15_ui_template_resource_resolver_typed_errors_static_passed_cargo_deferred",
        "review_f5_ui_template_resource_resolver_uses_typed_lookup_errors_before_diagnostics_boundary",
        "ui/template/asset/resource_ref/resolver.rs",
        "UiResourceLookupError",
        "UiResourceLookupResult",
        "ResourceLocatorError::EmptyLabel",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || resolver_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 UI resource resolver typed-error docs/status should record `{doc_anchor}`"
        );
    }
}
