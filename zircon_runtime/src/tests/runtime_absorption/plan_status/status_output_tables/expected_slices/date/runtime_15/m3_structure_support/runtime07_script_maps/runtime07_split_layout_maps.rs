pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
