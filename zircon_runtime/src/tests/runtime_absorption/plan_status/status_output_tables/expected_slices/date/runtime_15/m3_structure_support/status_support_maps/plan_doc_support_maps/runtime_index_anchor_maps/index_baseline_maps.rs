pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 runtime index subplan map 01-15 sync"
        | "Runtime 15 M3 runtime index problem-row parser P01-P17 sync" => Some("2026-07-01"),
        _ => None,
    }
}

// Status: runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred.
// Guard: runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked.
// Status: runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred.
// Guard: runtime_15_runtime_index_subplan_map_covers_01_15_status_locked.
