pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 review-guard direct-assertion row-ownership guard child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 review-guard direct-assertion export-chain guard child split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
