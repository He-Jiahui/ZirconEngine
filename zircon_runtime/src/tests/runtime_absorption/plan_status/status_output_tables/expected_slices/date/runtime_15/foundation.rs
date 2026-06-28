pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 M5 production dead-code suppression global gate" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F12 dead-code review status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 F12 dead-code runtime/editor boundary status guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 F12 UI text edit-state dead-code suppression cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F1 native host callback panic guard" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 lock poison policy guard folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime lock poison guard child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 F2 lock poison recovery guard" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 production direct lock unwrap global gate" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 config store lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime devtools lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core handle diagnostics lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core handle time lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core handle states lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime task lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime profiling lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core handle registry lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime registration structure behavior layout split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime registration structure owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 root entries guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 plugin bridge table lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 native live-host bridge methods lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 navigation lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 dynamic API session lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 dynamic scene spawn task lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 scene ECS parallel executor lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core resource manager lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset project manager lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset worker pool lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 WGPU render framework lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 RHI WGPU render device lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 animation manager lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 input runtime manager lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 script VM registry lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 ZrVM real backend runtime lock poison recovery" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 graphics facade visibility note" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F5 scene property access typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 animation manager typed errors" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 F5 animation asset binary typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 profile export typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 gameplay host typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 script scene hook typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 VM plugin management policy typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 UI surface input effect typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 UI input surrounding-text error source" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 UI template resource resolver typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 UI asset document typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 export CLI typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 host reflection docs CLI typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 dynamic API session typed errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 F5 typed API residual typed errors" {
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
    } else if slice == "Runtime 15 F5 zshader definition typed errors" {
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
