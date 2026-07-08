pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 plugin-importer status-output guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard row-data aggregation guard child split" => Some("2026-07-05"),
        _ => None,
    }
}
