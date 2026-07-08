pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support plan-doc expected-slice maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status-support expected-slice map child split" => Some("2026-07-05"),
        "Runtime 15 M3 status output expected-slice legacy child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status-output expected-slice legacy guard body folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status output expected-slice legacy group child-owner split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 status output expected-slice guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 expected-slice module-layout guard body folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 production guard support row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 production guard runtime row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 production guard status-support priority row-data child split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 production guard status-support priority guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 structure-support expected-slice map child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 structure-support expected-slice row data folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 structure-convention warning cleanup" => Some("2026-07-01"),
        _ => None,
    }
}
