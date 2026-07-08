pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-group status-row-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group status-row-doc guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 child-group status-row-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group status-row-doc root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group status-row-doc source/status-map sync" => Some("2026-07-08"),
        _ => None,
    }
}
