pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
