pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 support Hub project-actions tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub runtime-state tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard" => {
            Some("2026-06-27")
        }
        _ => None,
    }
}
