pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07/script expected-slice maps folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
