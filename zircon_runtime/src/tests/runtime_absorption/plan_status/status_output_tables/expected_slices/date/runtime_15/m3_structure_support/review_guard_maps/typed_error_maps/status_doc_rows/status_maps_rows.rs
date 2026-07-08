pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error status-doc status maps child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error status-doc status maps status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc status maps status-current sources child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc status maps status-current sources guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout sources child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout sources guard folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
