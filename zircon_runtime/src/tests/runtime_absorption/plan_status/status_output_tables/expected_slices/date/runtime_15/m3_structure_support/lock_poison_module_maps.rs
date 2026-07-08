pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 lock poison policy route-owner split" {
        Some("2026-07-05")
    } else if slice == "Runtime 15 M3 lock-poison split-layout guard folder-backed split" {
        Some("2026-07-05")
    } else if slice == "Runtime 15 M3 asset/render/input lock-poison guard child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 runtime services lock-poison guard child-owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some("2026-06-28")
    } else if slice == "Runtime 15 M3 module-convention status row-data owner child split" {
        Some("2026-07-07")
    } else if slice == "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M3 module convention gate guard folder-backed split" {
        Some("2026-07-05")
    } else if slice == "Runtime 15 M3 module-convention guard source reconciliation" {
        Some("2026-07-07")
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
    } else if slice == "Runtime 15 M3 module convention zero-debt revalidation" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some("2026-06-27")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("2026-06-23")
    } else {
        None
    }
}
