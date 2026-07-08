pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output expected-slice maps split" => Some("2026-06-23"),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard body folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard body budgets folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice maps guard-body route mounts folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard body folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard-body route mounts folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard route metadata child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard sources folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget route metadata child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget source inventory folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 status output expected-slice guard maps child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status-output expected-slice guard maps folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 foundation expected-slice maps guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 naming-boundary render-graphics map rows guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 foundation expected-slice maps status mirrors folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 naming-boundary expected-slice sources folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split" => {
            Some("2026-06-25")
        }
        _ => None,
    }
}
