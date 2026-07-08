pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings status-doc status-mirror child-owner split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc map-source sync" => Some("2026-07-07"),
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings status-doc source anchors folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings status-doc status anchors folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc child-anchor list child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings status-doc child-anchor route folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings status-doc root inventory child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split" => {
            Some("2026-07-05")
        }
        _ => None,
    }
}
