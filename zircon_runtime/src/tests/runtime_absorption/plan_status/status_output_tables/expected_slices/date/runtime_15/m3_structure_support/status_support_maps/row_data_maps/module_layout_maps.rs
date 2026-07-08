pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 module-layout status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout source/status-map sync" => Some("2026-07-08"),
        "Runtime 15 M3 status output row-data module-layout status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 module-layout status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout status-doc root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout status-doc source/status-map sync" => Some("2026-07-08"),
        "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout child-summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 module-layout child-summary root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout child-summary owner-budget guard child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary milestone-groups child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary foundation-review child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary source/status-map sync" => Some("2026-07-08"),
        "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout child-summary status-doc guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 module-layout child-summary status-doc status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary status-doc source/status-map sync" => {
            Some("2026-07-08")
        }
        _ => None,
    }
}
