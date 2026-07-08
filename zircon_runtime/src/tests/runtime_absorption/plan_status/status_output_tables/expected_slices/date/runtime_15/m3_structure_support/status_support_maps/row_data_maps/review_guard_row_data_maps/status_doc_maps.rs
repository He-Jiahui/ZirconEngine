pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard row-data status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard row-data status-doc source reconciliation" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard typed-error status-doc row-data folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error status-doc guard folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
