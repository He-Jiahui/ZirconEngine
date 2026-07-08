pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget child-source current-route sync" => {
            Some("2026-07-08")
        }
        "Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split" => Some("2026-07-01"),
        _ => None,
    }
}
