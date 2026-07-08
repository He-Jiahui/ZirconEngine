pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard typed-error expected-slice map child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice map rows folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice route metadata child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice route metadata folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice guard body folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
