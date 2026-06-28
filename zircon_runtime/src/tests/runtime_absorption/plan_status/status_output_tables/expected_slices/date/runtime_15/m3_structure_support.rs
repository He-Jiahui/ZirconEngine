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
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 module convention gate output contract" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention non-render debt guard" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 render-scoped migration debt handoff gate" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention gate audit-clear status mirror" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 code review findings test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 review top-row status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D7 core workspace dependency inheritance guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D8 runtime registration builder original evidence paths" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D9 editor/runtime mirror consumer guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D5 editor authoring macro consumer guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 typed-error convergence guard child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 input manager test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI shared core guard child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 UI shared core input visibility child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI shared core scroll mutation child folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI shared core layout surface child folder split" {
        Some("2026-06-24")
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
    } else if slice == "Runtime 15 M3 UI runtime input reply table pointer route folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI runtime input reply route guard child-owner split" {
        Some("2026-06-25")
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
    } else if slice == "Runtime 15 M3 asset project example vampire test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset artifact store test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset mesh test root split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset UI test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 asset pipeline manager test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 no oversized test files global gate" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM hot-reload coordinator test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 native live-host tests folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 native plugin loader real fixture test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 extension registry bridge test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 manifest contributions test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
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
    } else if slice == "Runtime 15 M3 test file budget root-layout child split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 foundation row data split" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M2 row data split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 support Hub project-actions tests child-owner split" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 support Hub runtime-state tests child-owner split" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split"
    {
        Some("2026-06-27")
    } else if slice
        == "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split"
    {
        Some("2026-06-27")
    } else if slice
        == "Runtime 15 M3 editor retained-host pane data conversion projection owner guard"
    {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 production file budget core runtime guard split" {
        Some("2026-06-23")
    } else if slice
        == "Runtime 15 M3 render shader template assembly guard support child-owner split"
    {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 shader prewarm manifest guard child-owner split" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M4 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output expected-slice maps split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 status output expected-slice guard maps child-owner split" {
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M3 status output expected-slice top-level map support child-owner split"
    {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M3 row data split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget root-layout status scan child split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 test file budget root-layout folder-backed guard child split"
    {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split"
    {
        Some("2026-06-25")
    } else if slice == "Runtime 15 M3 test file budget parent guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 historical oversized test roots closeout" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset test-budget guard child-owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI asset test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 UI asset surface index test folder split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 UI asset MUI web form style test folder split" {
        Some("2026-06-24")
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
    } else if slice == "Runtime 15 M3 status output expected-slice guard child-owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 D12 runtime helper export macro review sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D1 capability single-source review sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D10 animation/physics bridge call migration" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D13 importer manifest parity guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 P0/DX priority D13 parity sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D13 importer top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" {
        Some("2026-06-28")
    } else {
        None
    }
}
