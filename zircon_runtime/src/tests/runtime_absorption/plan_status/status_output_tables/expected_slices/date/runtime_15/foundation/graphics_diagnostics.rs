pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 graphics facade visibility note" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 M1 graphics facade visibility review findings mirror" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("2026-06-22")
    } else {
        None
    }
}
