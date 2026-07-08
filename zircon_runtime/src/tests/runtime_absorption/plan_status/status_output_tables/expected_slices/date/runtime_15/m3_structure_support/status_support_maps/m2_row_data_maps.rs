pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output Runtime 15 M2 row data split" => Some("2026-06-29"),
        "Runtime 15 M3 M2 row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 M2 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 M2 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 M2 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 M2 row-data source/status-map sync" => Some("2026-07-08"),
        _ => None,
    }
}
