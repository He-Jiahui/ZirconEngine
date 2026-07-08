pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings direct assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings render direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings render direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings direct assertions child-source sync" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
