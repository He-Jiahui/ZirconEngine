pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 production file budget guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output variable evidence anchors" => Some("2026-06-24"),
        "Runtime 15 M3 status output evidence anchors guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 evidence anchors status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 evidence anchors root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 evidence anchors source/status-map sync" => Some("2026-07-08"),
        _ => None,
    }
}
