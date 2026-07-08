pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 typed-error moved-guard absence parent-backflow child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error moved-guard absence root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error moved-guard absence child-owner route split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
