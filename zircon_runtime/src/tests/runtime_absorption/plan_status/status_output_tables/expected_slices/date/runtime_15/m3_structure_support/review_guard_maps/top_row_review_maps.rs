pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review top-row status row-data child-owner split" => Some("2026-06-28"),
        "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D7 core workspace dependency inheritance guard" => Some("2026-06-28"),
        "Runtime 15 M3 D8 runtime registration builder original evidence paths" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" => Some("2026-06-28"),
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D9 editor/runtime mirror consumer guard" => Some("2026-06-28"),
        "Runtime 15 M3 D5 editor authoring macro consumer guard" => Some("2026-06-28"),
        "Runtime 15 M3 D12 runtime helper export macro review sync" => Some("2026-06-28"),
        "Runtime 15 M3 D1 capability single-source review sync" => Some("2026-06-28"),
        "Runtime 15 M3 D10 animation/physics bridge call migration" => Some("2026-06-28"),
        "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" => Some("2026-06-28"),
        "Runtime 15 M3 D13 importer manifest parity guard" => Some("2026-06-28"),
        "Runtime 15 M3 P0/DX priority D13 parity sync" => Some("2026-06-28"),
        "Runtime 15 M3 D13 importer top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" => Some("2026-06-28"),
        _ => None,
    }
}
