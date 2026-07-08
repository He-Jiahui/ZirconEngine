pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime Cargo attempt status anchor sync" => {
            Some("runtime_15_runtime_cargo_attempt_status_anchor_sync_static_passed_cargo_deferred")
        }
        _ => None,
    }
}

// Guard: runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked.
