pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS schedule conflict graph child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene render extract test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene asset integration test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene world basics test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene property paths test folder split" {
        Some("2026-06-24")
    } else {
        None
    }
}
