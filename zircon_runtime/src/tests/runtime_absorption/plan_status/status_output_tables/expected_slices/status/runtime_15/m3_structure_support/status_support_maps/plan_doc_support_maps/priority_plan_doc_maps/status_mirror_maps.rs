pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs status-mirror child split" => Some(
            "runtime_15_priority_plan_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
