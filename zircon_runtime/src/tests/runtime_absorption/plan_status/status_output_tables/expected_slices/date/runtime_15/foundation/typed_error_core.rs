pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F5 scene property access typed errors" {
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
    } else if slice == "Runtime 15 F5 shader prewarm args typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm manifest merge typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm manifest read typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm report output typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm permutation registry typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm resource registry typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm asset-root scan typed errors" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 shader prewarm CLI typed-error sweep" {
        Some("2026-06-29")
    } else if slice == "Runtime 15 F5 dynamic API session typed errors" {
        Some("2026-06-27")
    } else {
        None
    }
}
