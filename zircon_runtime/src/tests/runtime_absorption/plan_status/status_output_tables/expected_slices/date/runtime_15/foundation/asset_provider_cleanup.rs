pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F5 typed API residual typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 fixed world mutation typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 asset authoring typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 navigation asset typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 font asset typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 sound asset typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F7 artifact cache JSON number typed errors" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 F5 zshader v2 user definition migration" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 asset meta typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 texture loader typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 mesh loader typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F8 texture descriptor typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F13 provider registration shared owner" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F13 provider update shared stats owner" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F13 provider feedback shared payload owner" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F13 provider prepare input shared frame owner" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F13 full provider boilerplate audit" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 runtime-owned dead-code suppression cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 script host value descriptor dead-code cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 script reflection macro fixture dead-code cleanup" {
        Some("2026-06-23")
    } else {
        None
    }
}
