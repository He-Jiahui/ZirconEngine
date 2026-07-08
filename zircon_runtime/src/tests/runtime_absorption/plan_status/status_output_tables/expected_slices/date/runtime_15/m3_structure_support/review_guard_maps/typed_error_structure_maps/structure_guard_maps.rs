pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 typed-error structure guard root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error child-ownership guard folder-backed split" => Some("2026-07-04"),
        "Runtime 15 M3 typed-error child-ownership root inventory child split" => {
            Some("2026-07-04")
        }
        _ => None,
    }
}
