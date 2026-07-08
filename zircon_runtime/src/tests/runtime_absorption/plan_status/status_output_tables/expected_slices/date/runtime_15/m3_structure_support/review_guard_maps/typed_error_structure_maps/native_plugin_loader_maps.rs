pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error native plugin loader routes child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error native plugin loader routes source helper child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error native plugin loader source helper child split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
