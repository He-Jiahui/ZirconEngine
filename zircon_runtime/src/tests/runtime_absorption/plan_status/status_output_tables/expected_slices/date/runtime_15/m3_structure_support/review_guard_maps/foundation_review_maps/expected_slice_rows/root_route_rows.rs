pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard expected-slice root route metadata child split"
        | "Runtime 15 M3 review-guard expected-slice root guard body folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice root guard body child ownership folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice root guard body route mounts folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice root route metadata guard folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice root route metadata route mounts folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice root route metadata status-mirror guard folder-backed split"
        | "Runtime 15 M3 review-guard root source inventory folder-backed split"
        | "Runtime 15 M3 review-guard structure row data folder-backed split" => Some("2026-07-06"),
        _ => None,
    }
}
