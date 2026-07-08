pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard expected-slice maps folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 review-guard expected-slice route metadata child split"
        | "Runtime 15 M3 review-guard expected-slice route metadata guard folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice route metadata budgets folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice route metadata route mounts folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice route metadata route mounts folder-backed guard body split"
        | "Runtime 15 M3 review-guard expected-slice route metadata status mirrors folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice route child sources folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice route metadata source constants folder-backed split"
        | "Runtime 15 M3 review-guard source structure paths folder-backed split"
        | "Runtime 15 M3 review-guard expected-slice guard body folder-backed split" => {
            Some("2026-07-06")
        }
        _ => None,
    }
}
