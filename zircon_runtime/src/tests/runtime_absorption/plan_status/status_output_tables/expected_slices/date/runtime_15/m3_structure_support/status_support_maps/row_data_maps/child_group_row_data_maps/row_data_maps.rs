pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups exports child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups inventory row-data child split" => Some("2026-07-05"),
        "Runtime 15 M3 child-groups owner-path budget groups folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 child-groups folder-backed source reconciliation" => Some("2026-07-07"),
        _ => None,
    }
}
