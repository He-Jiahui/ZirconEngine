pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
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
    } else if slice == "Runtime 15 M3 asset project zmeta current 12-test guard sync" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 asset project manager current 11-test guard sync" {
        Some("2026-07-03")
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
    } else if slice == "Runtime 15 M3 render product mesh-cache morph tests child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 UI text layout folder-backed owner split" {
        Some("2026-07-03")
    } else {
        None
    }
}
