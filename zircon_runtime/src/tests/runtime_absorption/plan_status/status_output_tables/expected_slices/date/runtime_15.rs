pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 graphics facade visibility note" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("2026-06-22")
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
    } else if slice == "Runtime 15 M4 core runtime service-list owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU command validation render-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface render/setup owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface geometry test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 material asset value/readiness helper owner split" {
        Some("2026-06-23")
    } else if slice
        == "Runtime 15 M4 core runtime render-stats graph execution-resources owner split"
    {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene fixed light reflection write-field owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world property-access physics write owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world property-access physics entry owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world project I/O mesh owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI text layout engine visual-order owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI layout arrange grid/masonry owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI template MUI X DataGrid class owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template document validation owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template style slot-contract owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI v2 style runtime-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI accessibility extract state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI component catalog editor-showcase helper owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split"
    {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI component state-reducer tree view editing owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface event-routing owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface property mutation metadata dirty owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface render feedback command/color owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface table column helper owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F12 offscreen target texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 render backend state owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu texture resource owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu material uniform owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu mesh order signature cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu model identity cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 post-process LUT texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 output target texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 material runtime capture seed cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 resource streamer diagnostics accessor cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 resource streamer resolve texture id cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 particle GPU readback output accessor cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 advanced plugin output test accessor cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 code review findings test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI layout slots test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI surface-frame authority test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI surface dirty domains test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI material layout test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI template test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI component catalog test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI boundary test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI component state test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI component state keyboard test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 RHI command list test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 RHI device contract test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset pack test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset facade test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project zmeta test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project flow sample test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("2026-06-23")
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
    } else if slice == "Runtime 15 M3 test file budget root-layout child split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 production file budget core runtime guard split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M4 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output expected-slice maps split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M3 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget root-layout status scan child split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 historical oversized test roots closeout" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset test-budget guard child-owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI asset test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI asset MUI X web style test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI asset MUI web style test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI taffy layout pass test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime window input pump test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime window event ABI child folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget root-layout UI child split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI widget text input keyboard test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI focus navigation test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input manager test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI runtime input ownership test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 production file budget guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 status output variable evidence anchors" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 status output M3 row data child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 status output row-data guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 status output expected-slice legacy child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 status output expected-slice legacy group child-owner split" {
        Some("2026-06-24")
    } else {
        None
    }
}
