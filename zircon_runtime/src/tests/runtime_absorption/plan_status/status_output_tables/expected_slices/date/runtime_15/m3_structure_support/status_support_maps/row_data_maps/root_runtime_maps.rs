pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support expected-slice row-data owner folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support row-data expected-slice maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support row-data route expected-slice guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status output M3 row data child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output row-data guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output Runtime 15 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 Runtime 15 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 row-data row-ownership child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 row-data source/status-map sync" => Some("2026-07-08"),
        "Runtime 15 M3 status-support anchor mirror child-owner split" => Some("2026-07-06"),
        _ => None,
    }
}
