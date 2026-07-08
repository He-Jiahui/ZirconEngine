pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 P0 robustness review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 P0 robustness structure guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 P0 robustness root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 P0 route ownership guard child split" => Some("2026-07-05"),
        "Runtime 15 M3 P0 source status-map reconciliation" => Some("2026-07-07"),
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 P0 native fixture leaf-owner root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 P0 native fixture source status-map reconciliation" => Some("2026-07-07"),
        _ => None,
    }
}
