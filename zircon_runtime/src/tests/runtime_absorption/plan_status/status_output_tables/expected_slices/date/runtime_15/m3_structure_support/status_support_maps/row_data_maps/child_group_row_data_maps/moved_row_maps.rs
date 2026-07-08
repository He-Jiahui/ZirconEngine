pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-group moved-row status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group moved-row root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}
