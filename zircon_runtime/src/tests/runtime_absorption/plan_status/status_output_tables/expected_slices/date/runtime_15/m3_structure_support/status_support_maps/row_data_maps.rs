pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output M3 row data child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output row-data guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 module-layout status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output row-data module-layout status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 module-layout status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 module-layout status-doc root inventory child split" => Some("2026-07-04"),
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
        "Runtime 15 M3 status output review-guard row-data guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard moved-row guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard moved-row status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard moved-row root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard moved-row code-review rows child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard code-review row-data root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 plugin-importer status-output guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard row-data status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 foundation row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 foundation-guards row-data root inventory child split" => {
            Some("2026-07-04")
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
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups status-doc root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups exports child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group status-row-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group status-row-doc guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 child-group status-row-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group status-row-doc root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 lock-poison status row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 lock-poison status row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 scene-script row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 scene-script row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 scene-script row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 scene-script row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-group moved-row status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group moved-row root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}
