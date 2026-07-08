pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 M1 graphics facade visibility review findings mirror" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 M5 production dead-code suppression global gate" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F12 dead-code review status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 F12 dead-code runtime/editor boundary status guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 F12 production dead-code current-state wording cleanup" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 F12 UI text edit-state dead-code suppression cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F1 native host callback panic guard" {
        Some("2026-06-27")
    } else {
        None
    }
}
