pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard foundation status-date maps folder-backed split"
        | "Runtime 15 M3 review-guard foundation status-date map guard folder-backed split"
        | "Runtime 15 M3 review-guard foundation route-mount guard folder-backed split"
        | "Runtime 15 M3 review-guard foundation status-mirror guard folder-backed split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
