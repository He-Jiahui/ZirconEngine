pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 priority plan docs owner-guard row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split" => Some("2026-07-03"),
        _ => None,
    }
}
