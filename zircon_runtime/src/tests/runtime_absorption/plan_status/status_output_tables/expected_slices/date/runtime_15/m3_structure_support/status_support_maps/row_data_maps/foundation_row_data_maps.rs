pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 foundation row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation row-data source/status-map sync" => Some("2026-07-08"),
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation-guards row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation-guards runtime-structure row-data child split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 foundation-guards runtime-structure guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 foundation row-data status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 foundation row-data row-count child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation row-data status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        _ => None,
    }
}
