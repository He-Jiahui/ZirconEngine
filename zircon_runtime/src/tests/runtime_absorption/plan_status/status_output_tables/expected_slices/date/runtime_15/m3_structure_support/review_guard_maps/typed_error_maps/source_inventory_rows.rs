pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error source inventory guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 typed-error source inventory guard folder-backed split" => Some("2026-07-04"),
        "Runtime 15 M3 typed-error source inventory source helper child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory child sources folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child sources structure guard child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child inventory folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child inventory status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory metadata child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory metadata status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory delegation child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory delegation folder-backed child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
