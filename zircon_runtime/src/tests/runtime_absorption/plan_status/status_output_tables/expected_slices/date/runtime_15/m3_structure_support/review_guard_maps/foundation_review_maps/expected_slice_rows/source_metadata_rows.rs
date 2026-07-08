pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard source metadata folder-backed split" => Some("2026-07-06"),
        _ => None,
    }
}
