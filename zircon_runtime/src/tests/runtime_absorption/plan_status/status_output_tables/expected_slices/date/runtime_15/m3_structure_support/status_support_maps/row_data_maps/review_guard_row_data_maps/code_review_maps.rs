pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard code-review row-data root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard code-review export/status source reconciliation" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard typed-error structure-assertions row-data folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error structure-assertions guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error structure row-data guard folder-backed split" => Some("2026-07-07"),
        _ => None,
    }
}
