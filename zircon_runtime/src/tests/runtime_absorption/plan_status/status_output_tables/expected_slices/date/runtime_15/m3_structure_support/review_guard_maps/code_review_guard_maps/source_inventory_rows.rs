pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings source inventory child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings source inventory folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings source inventory status-mirror child-owner split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings source inventory map-source sync" => Some("2026-07-07"),
        _ => None,
    }
}
