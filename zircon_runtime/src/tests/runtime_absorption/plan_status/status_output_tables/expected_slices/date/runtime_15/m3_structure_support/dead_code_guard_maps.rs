pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup"
    {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 provider boilerplate guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 runtime dead-code guard child-owner split" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code documentation anchor cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code module-gate status wording cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 runtime dead-code production-gate status wording cleanup" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("2026-06-23")
    } else {
        None
    }
}
