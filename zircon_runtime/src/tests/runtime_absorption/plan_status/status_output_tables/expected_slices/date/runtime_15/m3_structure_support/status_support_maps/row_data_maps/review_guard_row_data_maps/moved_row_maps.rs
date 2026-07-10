pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review-guard moved-row guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard moved-row status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard moved-row root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard moved-row code-review rows child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard moved-row code-review rows route metadata child split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
