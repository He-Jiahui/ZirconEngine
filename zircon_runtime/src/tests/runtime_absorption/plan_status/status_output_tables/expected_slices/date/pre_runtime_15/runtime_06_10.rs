pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 10 F18 asset manager resolution return shape" {
        Some("2026-06-22")
    } else if slice == "Runtime 08 F17 entity path lookup verb rename" {
        Some("2026-06-22")
    } else if matches!(
        slice,
        Some("Runtime 09 UI architecture 镜像文档守卫")
            | "Runtime 09 surface default interaction fallback rename"
            | "Runtime 07 ECS frame diagnostics aggregation"
            | "Runtime 07 QueryState frame auto-collection"
            | "Runtime 07 ChangeDetection frame auto-collection"
            | "Runtime 07 QueryState iterator lifetime guard"
            | "Runtime 07 FPS gate support unblock"
            | "Runtime 07 profiling build tooling"
            | "Runtime 07 extract rebuild cache"
            | "Runtime 07 extract cache hit/miss diagnostics"
            | "Runtime 07 asset worker frame sampler"
            | "Runtime 04 worker-pool manager frame sampler entry"
            | "Runtime 07 asset worker manager sampler entry"
            | "Runtime 07 artifact cache payload owner split"
            | "Runtime 07 render product diagnostics owner split"
            | "Runtime 07 animation scene frame diagnostics"
            | "Runtime 07 QueryState cache owner performance audit sync"
            | "Runtime 07 virtual geometry debug snapshot owner split"
            | "Runtime 07 owner-budget 42-hotspot 漂移同步"
            | "Runtime 07 owner-budget current doc mirror fix"
            | "Runtime 08 QueryState cache owner split"
            | "Runtime 10 UI contract duplicate public types cleanup"
            | "Runtime 10 UI v2 contract sync"
            | "Runtime 12 action context routing"
            | "Runtime 12 gamepad bridge source guard event-owner sync"
            | "Runtime 12 action axis value bindings"
            | "Runtime 12 gamepad axis transition edges"
            | "Runtime 12 consumed gamepad axis arbitration"
            | "Runtime 12 input recording/replay"
            | "Runtime 12 action map config source"
            | "Runtime 12 action manager registration path"
            | "Runtime 11 scheduler wait_all 同步点"
            | "Runtime 02 root graphics alias block removal"
            | "Runtime 02 rhi_wgpu root backend private cutover"
            | "Runtime 02 builtin root facade cutover"
            | "Runtime 05 texture importer DDS caps policy wording"
    ) {
        Some("2026-06-17")
    } else if matches!(
        slice,
        Some("Runtime 07 owner-budget 38-hotspot 回漂同步")
            | "Runtime 07 owner-budget 39-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 再同步"
    ) {
        Some("2026-06-15")
    } else if slice == "Runtime 07 owner-budget 30-hotspot current audit sync"
        || slice == "Runtime 07 profile counter hotspot export"
        || slice == "Runtime 11 panic-safe handle completion"
        || slice == "Runtime 05 dynamic scene root scene owner split"
        || slice == "Runtime 05 dynamic scene spawn task owner split"
        || slice == "Runtime 05 dynamic scene value conversion owner split"
        || slice == "Runtime 05 dynamic scene entity declaration owner split"
        || slice == "Runtime 05 dynamic scene scene-asset bridge owner split"
        || slice == "Runtime 05 dynamic scene document serialization owner split"
        || slice == "Runtime 05 dynamic scene patch preview API"
        || slice == "Runtime 05 dynamic scene patch preview status guard"
        || slice == "Runtime 05 dynamic scene patch preview resource preflight details status guard"
        || slice == "Runtime 05 dynamic scene patch preview resource ensure creation status guard"
        || slice
            == "Runtime 05 dynamic scene patch preview component type install details status guard"
        || slice
            == "Runtime 05 dynamic scene patch preview component type install counts status guard"
        || slice == "Runtime 05 dynamic scene patch preview reflection preflight status guard"
        || slice == "Runtime 05 dynamic scene patch preview component workload status guard"
        || slice == "Runtime 05 dynamic scene patch preview remap status guard"
        || slice == "Runtime 05 dynamic scene session owner-tree guard"
        || slice == "Runtime 05 dynamic scene root owner-tree guard"
        || slice == "Runtime 05 status-output Runtime 05 row-data family split"
        || slice == "Runtime 05 status-output audit-metadata owner split"
        || slice == "Runtime 05 status-output Runtime 14 row-data family split"
        || slice == "Runtime 05 status-output Runtime 07 row-data family split"
        || slice == "Runtime 05 status-output Runtime 09 row-data family split"
        || slice == "Runtime 05 status-output Runtime 10 row-data family split"
        || slice == "Runtime 05 status-output Runtime 12 row-data family split"
        || slice == "Runtime 05 status-output support-structure owner split"
        || slice == "Runtime 05 status-output scene-closeout owner split"
        || slice == "Runtime 05 status-output cargo-gates owner split"
        || slice == "Runtime 05 status-output status/date owner split"
        || slice == "Runtime 05 status-output Runtime 07 owner-budget row"
        || slice == "Runtime 05 plan-status owner-budget current mirror fix"
        || slice == "Runtime 05 status-output Runtime 01-04 row-data group split"
        || slice == "Runtime 05 status-output Runtime 06-09 row-data group split"
        || slice == "Runtime 05 status-output Runtime 10-13 row-data group split"
        || slice == "Runtime 05 editor_projection residual guard verdict"
        || slice == "Runtime 05 scene:: diagnostic matrix source anchors"
    {
        Some("2026-06-20")
    } else if slice == "Runtime 10 host-request payload ABI boundary" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 Dynamic API current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 Dynamic API test boundary Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 10 dynamic_api_session Cargo 验证窗口探测" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 runtime diagnostics profile-control snapshot" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 diagnostics inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 host-request inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 UI contract inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 validation inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 session lifecycle inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 failure boundary inventory split" {
        Some("2026-06-20")
    } else if slice == "Runtime 10 ABI source inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 10 runtime API Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 10 dynamic runtime API Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 10 dynamic input mouse-wheel event owner guard" {
        Some("2026-06-21")
    } else if slice == "Runtime 10 Vampire W input real-backend gate" {
        Some("2026-06-21")
    } else if slice == "Runtime 09 UI architecture Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 06 plugin surface/lifecycle Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 06 native plugin public-surface Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor builder scaffold" {
        Some("2026-06-22")
    } else if slice == "Runtime 06 F8 first-party RuntimePluginDescriptor builder migration" {
        Some("2026-06-22")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor test fixture builder migration" {
        Some("2026-06-22")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public-field convergence" {
        Some("2026-06-22")
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public constructor retired" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 Performance hotpath Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 07 scene/EventBus poison-safe locks" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render submit source-extract sharing" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render submit viewport/provider errors" {
        Some("2026-06-27")
    } else if slice == "Runtime 07 render camera-loop descriptor submissions" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render camera-loop borrowed sequence resolution" {
        Some("2026-06-27")
    } else if slice == "Runtime 07 render camera-loop source view restore narrowing" {
        Some("2026-06-27")
    } else if slice == "Runtime 07 render camera-loop post-process source restore narrowing" {
        Some("2026-06-27")
    } else if slice == "Runtime 07 render camera-loop VG/HGI conditional source restore" {
        Some("2026-06-28")
    } else if slice == "Runtime 07 render camera-loop single-child source-state capture skip" {
        Some("2026-06-28")
    } else if slice == "Runtime 07 render camera-loop source payload slot ownership" {
        Some("2026-06-28")
    } else if slice == "Runtime 07 render camera-loop frame terminal move" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render submit feedback sideband owned merge" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render prepared sideband frame owner move" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render direct runtime-frame streaming camera loop" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render generated camera-loop shared extract" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render shared effective extract frame source" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render direct runtime-frame shared context extract" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render VG debug overlay frame override" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render direct runtime-frame trace export" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 render submit effective extract projection" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 F16 compiled-scene split status guard" {
        Some("2026-06-28")
    } else if slice == "Runtime 08 F5 world typed mutation errors" {
        Some("2026-06-22")
    } else if slice == "Runtime 08 F5 dynamic component typed errors" {
        Some("2026-06-22")
    } else if slice == "Runtime 07 Performance hotpath inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 08 ECS 数据面 current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS source/test inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 08 ECS anchor inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 08 ECS markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 08 QueryState Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 08 ECS event owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS message owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS resource store owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS resource identity owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS component identity owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS entity identity owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS archetype owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS component storage owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS component storage private re-export cleanup" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS observer owner folder split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS commands facade owner split" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS command Cargo 验证窗口探测" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS entity Cargo 验证窗口探测" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS data owner-tree guard" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS change detection owner-tree guard" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ECS root leaf owner guard" {
        Some("2026-06-20")
    } else if slice == "Runtime 08 ecs_events_messages Cargo 验证窗口探测" {
        Some("2026-06-20")
    } else {
        None
    }
}
