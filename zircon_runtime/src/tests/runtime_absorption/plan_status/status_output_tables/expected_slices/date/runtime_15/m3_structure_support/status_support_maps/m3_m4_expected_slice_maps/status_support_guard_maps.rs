pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support top-level support row data folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support route metadata row data folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support status support maps row data folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support route guard rows row-data owner child split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard structure row data folder-backed split" => Some("2026-07-07"),
        "Runtime 15 M3 status-support M3/M4 expected-slice maps folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support expected-slice parent maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support expected-slice owner paths folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support expected-slice owner paths guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support expected-slice guard route metadata child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status-support expected-slice route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status-support expected-slice route metadata status mirrors folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status-support expected-slice guard body folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status-support expected-slice guard body status mirrors folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status-support parent-route expected-slice guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support row-data route expected-slice guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support plan-doc route expected-slice guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support review-guard row-data route expected-slice guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support runtime-index anchor row-data child split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
