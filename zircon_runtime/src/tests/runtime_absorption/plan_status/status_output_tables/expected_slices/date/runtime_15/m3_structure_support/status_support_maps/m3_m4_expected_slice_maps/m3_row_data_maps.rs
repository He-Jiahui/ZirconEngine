pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output Runtime 15 M3 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 M3 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 Runtime 15 M3 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 M3 row-data root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}
