pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings structure guard child-owner split" => {
            Some("2026-06-29")
        }
        "Runtime 15 M3 code review findings structure guard children folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings structure guard children budget-status child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings structure guard children root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings structure guard children source-map sync" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 structure guard plugin-importer child split" => Some("2026-07-05"),
        "Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings structure guard typed-error child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings structure guard typed-error folder-backed split" => {
            Some("2026-07-03")
        }
        _ => None,
    }
}
