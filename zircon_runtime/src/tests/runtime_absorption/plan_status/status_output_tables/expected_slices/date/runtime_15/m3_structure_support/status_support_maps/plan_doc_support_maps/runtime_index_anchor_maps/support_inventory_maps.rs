pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 plan-status support inventory review sync" => Some("2026-07-01"),
        _ => None,
    }
}

// Status: runtime_15_plan_status_support_inventory_review_sync_static_passed_cargo_deferred.
// Guard: runtime_architecture_review_documents_all_absorption_guards.
