pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M4 core runtime service-list owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU command validation render-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface render/setup owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface geometry test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 RHI device handle owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 dynamic API session profile owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 dynamic API session registry owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 native host API adapter tests owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 material asset value/readiness helper owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 material asset management record owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 asset artifact cache UI document owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 mesh asset management record owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 asset project scan/import source collection owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 glTF labeled material subasset owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 texture descriptor settings parser owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 scene world render light collection owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 scene component lighting/post-process owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 render shader template assembly guard WGSL contracts split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M4 core runtime render-stats graph execution-resources owner split"
    {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 render-stats product diagnostics test owner split" {
        Some("2026-06-24")
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
    } else if slice == "Runtime 15 M4 UI dispatch input manager test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI template MUI X DataGrid class owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template document validation owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template style slot-contract owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI v2 style runtime-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI v2 style token-resolution owner split" {
        Some("2026-06-24")
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
    } else {
        None
    }
}
