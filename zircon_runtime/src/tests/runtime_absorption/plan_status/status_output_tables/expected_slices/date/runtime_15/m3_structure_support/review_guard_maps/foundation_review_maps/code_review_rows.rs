pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 code-review standalone harness current-path sync" => Some("2026-07-03"),
        _ => None,
    }
}
