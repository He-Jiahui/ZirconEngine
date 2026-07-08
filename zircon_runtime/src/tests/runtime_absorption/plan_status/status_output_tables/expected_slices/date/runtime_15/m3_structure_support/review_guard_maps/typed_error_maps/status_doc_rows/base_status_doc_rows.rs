pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error status-doc doc mirrors source helper child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc source helper child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error status-doc paths child split" => Some("2026-07-05"),
        _ => None,
    }
}
