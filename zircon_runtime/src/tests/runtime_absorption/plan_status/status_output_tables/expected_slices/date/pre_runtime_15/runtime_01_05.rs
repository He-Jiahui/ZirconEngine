pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if matches!(
        slice,
        Some("Runtime 02 root_entries guard-count current resync")
            | "Runtime 05 Runtime 02 root_entries count 状态表闭环"
            | "Runtime 06 native root re-export current mirror fix"
            | "Runtime 06 plugin::native hard-cutover"
            | "Runtime 06 fallback lifecycle failure tests"
            | "Runtime 06 fallback lifecycle Cargo 验证"
            | "Runtime 06 shader artifact cache real-backend unblock"
            | "Runtime 06 Vampire real-backend menu/retry focused validation"
            | "Runtime 06 Vampire HUD real-backend capture validation"
            | "Runtime 06 native loader test namespace migration"
            | "Runtime 06 V1/V2 ABI hard-cutover"
            | "Runtime 06 hot reload failure injection"
            | "Runtime 09 UI input route authority"
            | "Runtime 09 navigation legacy reply rename"
            | "Runtime 09 pointer legacy reply rename"
            | "Runtime 09 pointer capture fallback rename"
            | "Runtime 09 table row label fallback rename"
            | "Runtime 09 template component-name fallback rename"
            | "Runtime 09 property visibility flag rename"
            | "Runtime 09 responsive MUI visibility flag rename"
            | "Runtime 09 accessibility open-state fallback rename"
            | "Runtime 09 layout engine backend name cutover"
            | "Runtime 09 taffy bridge pass order"
            | "Runtime 09 virtualization scroll boundary"
            | "Runtime 09 template pipeline boundary"
            | "Runtime 11 graphics frustum rayon cutover"
            | "Runtime 13 Gameplay host predicate functions for real ZR VM"
            | "Runtime 05 status-output current anchor fix"
    ) {
        Some("2026-06-16")
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        Some("2026-06-15")
    } else if slice == "Runtime 05 scene/project serialization Markdown renderer split"
        || slice == "Runtime 05 scene/editor surface Markdown renderer split"
        || slice == "Runtime 05 non-network server Markdown renderer split"
        || slice == "Runtime 05 runtime naming Markdown renderer split"
        || slice == "Runtime 05 hard-cutover migration-smell Markdown renderer split"
        || slice == "Runtime M0 entry static dependencies Markdown renderer split"
        || slice == "Runtime M0 legacy standalone references Markdown renderer split"
        || slice == "Runtime M0 module inventory Markdown renderer split"
        || slice == "Runtime M0 plugin runtime gaps Markdown renderer split"
        || slice == "Runtime M0 large-file ownership Markdown renderer split"
        || slice == "Runtime 05 plan-status markdown direct import hard-cutover"
    {
        Some("2026-06-21")
    } else if slice == "Runtime 05 full scene compile-pass graphics-scene blocker" {
        Some("2026-06-20")
    } else if slice == "Runtime 05 render product streamer 2026-06-21 no-result diagnostic" {
        Some("2026-06-21")
    } else if slice == "Runtime 05 scene_asset 2026-06-21 no-result diagnostic" {
        Some("2026-06-21")
    } else if slice == "Runtime 05 ecs_query 2026-06-21 no-result diagnostic" {
        Some("2026-06-21")
    } else if slice == "Runtime 01 Tech-stack current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 01 Tech-stack 2026-07-01 current audit recheck" {
        Some("2026-07-01")
    } else if slice == "Runtime 01 Tech-stack inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 01 Tech-stack Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 01 Tech-stack SharedTextService 锚点同步" {
        Some("2026-07-01")
    } else if slice == "Runtime 02 F6 core resource registry typed errors" {
        Some("2026-06-22")
    } else if slice == "Runtime 02 core/root/generated current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 02 core/root/generated 2026-07-01 current audit recheck" {
        Some("2026-07-01")
    } else if slice == "Runtime 02 core/root/generated Markdown renderer split"
        || slice == "Runtime 02 generated-code Markdown renderer split"
        || slice == "Runtime 02 root-surface Markdown renderer split"
    {
        Some("2026-06-21")
    } else if slice == "Runtime 03 Schedule/frame-loop current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 03 Schedule/frame-loop 2026-07-01 current audit recheck" {
        Some("2026-07-01")
    } else if slice == "Runtime 03 Schedule/frame-loop inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 03 Schedule/frame-loop markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 03 Schedule/frame-loop session profile owner audit sync" {
        Some("2026-07-01")
    } else if slice == "Runtime 01 export build-plan directory materialization boundary"
        || slice == "Runtime 01 NativeDynamic materialization symlink boundary"
        || slice == "Runtime 01 export materialization dry-run preview"
        || slice == "Runtime 01 export materialization fatal preflight gate"
        || slice == "Runtime 01 editor native-aware fatal export early exit"
        || slice == "Runtime 01 editor native-aware discovery reuse"
        || slice == "Runtime 01 export ZIP archive materialization"
        || slice == "Runtime 04 asset worker request entry hard-cutover"
        || slice == "Runtime 04 Asset pipeline current audit recheck"
    {
        Some("2026-06-20")
    } else if slice == "Runtime 04 Asset pipeline inventory split"
        || slice == "Runtime 04 Asset pipeline Markdown renderer split"
    {
        Some("2026-06-21")
    } else if slice == "Runtime 04 Asset pipeline 2026-07-01 current audit recheck" {
        Some("2026-07-01")
    } else if slice == "Runtime 04 artifact-store child owner audit sync" {
        Some("2026-07-01")
    } else if slice == "Runtime 04 F7 asset artifact/importer typed errors"
        || slice == "Runtime 04 F8 texture import settings apply API"
    {
        Some("2026-06-22")
    } else {
        None
    }
}
