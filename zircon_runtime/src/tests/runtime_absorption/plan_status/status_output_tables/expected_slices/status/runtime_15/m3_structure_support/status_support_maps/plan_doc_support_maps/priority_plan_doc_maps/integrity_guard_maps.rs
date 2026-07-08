pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs code-path integrity guard" => Some(
            "runtime_15_priority_plan_docs_code_path_integrity_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs test-path integrity guard" => Some(
            "runtime_15_priority_plan_docs_test_path_integrity_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs frontmatter status guard" => Some(
            "runtime_15_priority_plan_docs_frontmatter_status_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs frontmatter uniqueness guard" => Some(
            "runtime_15_priority_plan_docs_frontmatter_uniqueness_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs required header sections guard" => Some(
            "runtime_15_priority_plan_docs_required_header_sections_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs plan-source cross-link guard" => Some(
            "runtime_15_priority_plan_docs_plan_source_cross_link_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test listing guard" => Some(
            "runtime_15_priority_plan_docs_guard_test_listing_guard_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
