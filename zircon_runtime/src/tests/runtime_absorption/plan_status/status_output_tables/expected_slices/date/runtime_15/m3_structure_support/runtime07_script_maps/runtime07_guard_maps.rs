pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 07 submit-context guard child-owner split" => Some("2026-07-05"),
        "Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split" => Some("2026-07-05"),
        "Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split" => Some("2026-07-06"),
        "Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 scene/project guard child-owner split" => Some("2026-07-06"),
        "Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters child-owner split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
