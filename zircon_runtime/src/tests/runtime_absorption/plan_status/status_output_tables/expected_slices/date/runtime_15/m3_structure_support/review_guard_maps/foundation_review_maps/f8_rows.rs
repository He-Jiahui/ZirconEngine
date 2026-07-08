pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 F8 API convergence review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 F8 child-owner root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 F8 route ownership guard child split" => Some("2026-07-05"),
        "Runtime 15 M3 F8 child-owner source status-map reconciliation" => Some("2026-07-07"),
        "Runtime 15 M3 F8 descriptor review guard child-owner split" => Some("2026-06-30"),
        _ => None,
    }
}
