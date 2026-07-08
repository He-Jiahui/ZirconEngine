pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output Runtime 15 foundation row data split" => Some("2026-06-27"),
        "Runtime 15 M3 foundation row-data topic child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 foundation row-data 73-row docs sync" => Some("2026-07-01"),
        "Runtime 15 M3 foundation row-data status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 foundation row-data row-count child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation row-data status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 foundation row-data stale-count prose guard" => Some("2026-07-03"),
        "Runtime 15 M3 foundation row-data priority-doc frontmatter sync" => Some("2026-07-03"),
        "Runtime 15 M3 foundation row-data status-doc source/status-map sync" => Some("2026-07-08"),
        _ => None,
    }
}
