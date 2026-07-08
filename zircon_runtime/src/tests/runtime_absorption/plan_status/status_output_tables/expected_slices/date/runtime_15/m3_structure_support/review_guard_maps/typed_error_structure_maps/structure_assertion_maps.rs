pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error structure assertions source reconciliation" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 code review findings structure guard typed-error structure assertions folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error convergence mounts guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error convergence mounts root inventory child split" => {
            Some("2026-07-04")
        }
        _ => None,
    }
}
