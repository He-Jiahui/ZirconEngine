pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 plugin-importer DX structure guard root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer D1 capability single-source guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 plugin-importer DX source status-map reconciliation" => {
            Some("2026-07-07")
        }
        _ => None,
    }
}
