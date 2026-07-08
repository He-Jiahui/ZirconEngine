pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings folder-backed summary child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split" => {
            Some("2026-07-04")
        }
        _ => None,
    }
}
