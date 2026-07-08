pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support review-guard row-data expected-slice maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status output review-guard row-data guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data delegation guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard row-data delegation split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard row-data root child rows folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard row-data root child rows split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data budgets guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard row-data root paths folder-backed split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
