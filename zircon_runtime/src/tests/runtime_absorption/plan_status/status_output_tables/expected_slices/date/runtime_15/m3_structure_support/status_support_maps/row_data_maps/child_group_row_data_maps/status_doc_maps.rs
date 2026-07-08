pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups status-doc root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}
