pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure source/status-map sync" => Some("2026-07-07"),
        _ => None,
    }
}
