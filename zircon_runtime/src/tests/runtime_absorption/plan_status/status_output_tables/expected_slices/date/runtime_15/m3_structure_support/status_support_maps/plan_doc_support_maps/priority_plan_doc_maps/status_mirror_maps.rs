pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs status-mirror child split" => Some("2026-07-04"),
        _ => None,
    }
}
