pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error status-doc paths status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc root paths folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc root paths folder-backed guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc status-slices folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc status-slices folder-backed guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc paths status-current sources child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc paths status-current sources guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout sources child split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout sources guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout status mirrors guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout status mirrors status-current folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout status mirrors guard folder-backed split" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
