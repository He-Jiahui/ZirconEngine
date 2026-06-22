pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        "2026-06-22"
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        "2026-06-22"
    } else if slice == "Runtime 15 graphics facade visibility note" {
        "2026-06-22"
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        "2026-06-22"
    } else if slice == "Runtime 15 F13 provider registration shared owner" {
        "2026-06-22"
    } else if slice == "Runtime 15 F13 provider update shared stats owner" {
        "2026-06-22"
    } else if slice == "Runtime 15 F13 provider feedback shared payload owner" {
        "2026-06-22"
    } else if slice == "Runtime 15 F13 provider prepare input shared frame owner" {
        "2026-06-22"
    } else if slice == "Runtime 15 F13 full provider boilerplate audit" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 runtime-owned dead-code suppression cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 script host value descriptor dead-code cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 offscreen target texture owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 render backend state owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 gpu texture resource owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 gpu material uniform owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 gpu mesh order signature cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 gpu model identity cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 post-process LUT texture owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 output target texture owner cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 material runtime capture seed cleanup" {
        "2026-06-22"
    } else if slice == "Runtime 15 F12 resource streamer diagnostics accessor cleanup" {
        "2026-06-23"
    } else if slice == "Runtime 15 F12 resource streamer resolve texture id cleanup" {
        "2026-06-23"
    } else if slice == "Runtime 15 F12 particle GPU readback output accessor cleanup" {
        "2026-06-23"
    } else if slice == "Runtime 15 F12 advanced plugin output test accessor cleanup" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        "2026-06-23"
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        "2026-06-23"
    } else if slice == "Runtime 10 F18 asset manager resolution return shape" {
        "2026-06-22"
    } else if slice == "Runtime 08 F17 entity path lookup verb rename" {
        "2026-06-22"
    } else if matches!(
        slice,
        "Runtime 09 UI architecture 镜像文档守卫"
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
        "2026-06-17"
    } else if matches!(
        slice,
        "Runtime 02 root_entries guard-count current resync"
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
        "2026-06-16"
    } else if matches!(
        slice,
        "Runtime 07 owner-budget 38-hotspot 回漂同步"
            | "Runtime 07 owner-budget 39-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 再同步"
    ) {
        "2026-06-15"
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        "2026-06-15"
    } else if slice == "Runtime 14 animation family 28-file audit sync"
        || slice == "Runtime 14 navigation fallback runtime owner split"
        || slice == "Runtime 07 owner-budget 36-hotspot navigation split sync"
    {
        "2026-06-17"
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
        "2026-06-20"
    } else if slice == "Runtime 14 Module family guard anchors 审计同步" {
        "2026-06-15"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "2026-06-15"
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
        "2026-06-21"
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步"
        || slice == "Runtime 01 Tech-stack 行为测试锚审计同步"
        || slice == "Runtime 02 core/root/generated 镜像文档守卫"
        || slice == "Runtime 02 guard-test anchors 审计同步"
        || slice == "Runtime 10 Dynamic API 行为测试锚审计同步"
        || slice == "Runtime 10 dynamic_api_session 吸收守卫拆分"
        || slice == "Runtime 12 Input stack 行为测试锚审计同步"
        || slice == "Runtime 04 Asset pipeline 行为测试锚审计同步"
        || slice == "Runtime 08 ECS 行为测试锚审计同步"
        || slice == "Runtime 05 status-output Runtime 08 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 12 gamepad event-owner row"
        || slice == "Runtime 05 status-output Runtime 12 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 04 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 03 module-doc row"
        || slice == "Runtime 05 status-output Runtime 03 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 10 behavior-test row"
        || slice == "Runtime 05 plan-status output-anchor module split"
        || slice == "Runtime 05 plan-status output-anchor budget guard"
        || slice == "Runtime 05 status-output status/date helper split"
        || slice == "Runtime 05 status-output expected anchor split"
        || slice == "Runtime 05 status-output row-data group split"
        || slice == "Runtime 05 plan-status root module split"
        || slice == "Runtime 05 plan-status support inventory split"
        || slice == "Runtime 05 plan-status anchor inventory split"
        || slice == "Runtime 05 plan-status markdown renderer split"
        || slice == "Runtime 05 plan-status source helper split"
        || slice == "Runtime 05 status-output expected row data split"
        || slice == "Runtime 05 full scene closeout failed evidence"
        || slice == "Runtime 05 full scene closeout no-result recheck"
        || slice == "Runtime 05 scene:: failure support-first triage"
        || slice == "Runtime 05 scene:: lower-layer diagnostic matrix"
        || slice == "Runtime 05 serialization source folder-split guard sync"
        || slice == "Runtime 03 world bootstrap fixed-loop stage guard sync"
        || slice == "Runtime 05 cargo-gates early Runtime 03 split"
        || slice == "Runtime 05 cargo-gates early Runtime 01 split"
        || slice == "Runtime 05 cargo-gates early Runtime 02 split"
        || slice == "Runtime 05 cargo-gates early Runtime 04 split"
        || slice == "Runtime 05 cargo-gates early Runtime 06 split"
        || slice == "Runtime 05 cargo-gates early Runtime 08 split"
        || slice == "Runtime 05 cargo-gates early Runtime 07 split"
        || slice == "Runtime 05 cargo-gates late Runtime 10 split"
        || slice == "Runtime 05 cargo-gates late Runtime 11 split"
        || slice == "Runtime 05 cargo-gates late Runtime 12 split"
        || slice == "Runtime 05 cargo-gates late Runtime 13 split"
        || slice == "Runtime 05 cargo-gates late Runtime 14 split"
        || slice == "Runtime 05 status-output all-index-row coverage guard"
        || slice == "Runtime 03 Schedule/frame-loop 行为测试锚审计同步"
        || slice == "Runtime 08 First-stage event update guard"
    {
        "2026-06-15"
    } else if slice == "Runtime 05 full scene compile-pass graphics-scene blocker" {
        "2026-06-20"
    } else if slice == "Runtime 05 render product streamer 2026-06-21 no-result diagnostic" {
        "2026-06-21"
    } else if slice == "Runtime 05 scene_asset 2026-06-21 no-result diagnostic" {
        "2026-06-21"
    } else if slice == "Runtime 05 ecs_query 2026-06-21 no-result diagnostic" {
        "2026-06-21"
    } else if slice == "Runtime 11 JobSystem 2026-06-20 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 11 JobSystem core-min 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 11 JobSystem current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 11 JobSystem inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 11 JobSystem Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 11 worker wait-assist" {
        "2026-06-21"
    } else if slice == "Runtime 11 worker wait-assist core-min 验证窗口探测" {
        "2026-06-21"
    } else if slice == "Runtime 11 worker wait-assist core-min test binary 验证" {
        "2026-06-21"
    } else if slice == "Runtime 11 core-min test binary task/guard batch" {
        "2026-06-21"
    } else if slice == "Runtime 11 ecs_schedule source-guard lifetime anchor repair" {
        "2026-06-21"
    } else if slice == "Runtime 11 ecs_schedule core-min Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 tasks core-min Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 worker_pool core-min Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 rayon core-min Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 tasks default Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 worker_pool default Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 rayon default Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 ecs_schedule default Cargo 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 full-lib default Cargo closeout attempt" {
        "2026-06-21"
    } else if slice == "Runtime 11 core runtime full-lib triage recheck" {
        "2026-06-21"
    } else if slice == "Runtime 11 asset broader failure triage core-min 复验" {
        "2026-06-21"
    } else if slice == "Runtime 11 full-lib default after asset triage recheck" {
        "2026-06-21"
    } else if slice == "Runtime 11 full-lib default after graphics exposure retry" {
        "2026-06-21"
    } else if slice == "Runtime 12 cursor host requests" {
        "2026-06-20"
    } else if slice == "Runtime 10 host-request payload ABI boundary" {
        "2026-06-20"
    } else if slice == "Runtime 10 Dynamic API current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 10 Dynamic API test boundary Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 01 Tech-stack current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 01 Tech-stack inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 01 Tech-stack Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 10 dynamic_api_session Cargo 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 10 runtime diagnostics profile-control snapshot" {
        "2026-06-20"
    } else if slice == "Runtime 10 diagnostics inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 host-request inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 UI contract inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 validation inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 session lifecycle inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 failure boundary inventory split" {
        "2026-06-20"
    } else if slice == "Runtime 10 ABI source inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 10 runtime API Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 10 dynamic runtime API Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 10 dynamic input mouse-wheel event owner guard" {
        "2026-06-21"
    } else if slice == "Runtime 10 Vampire W input real-backend gate" {
        "2026-06-21"
    } else if slice == "Runtime 09 UI architecture Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 06 plugin surface/lifecycle Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 06 native plugin public-surface Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor builder scaffold" {
        "2026-06-22"
    } else if slice == "Runtime 06 F8 first-party RuntimePluginDescriptor builder migration" {
        "2026-06-22"
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor test fixture builder migration" {
        "2026-06-22"
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public-field convergence" {
        "2026-06-22"
    } else if slice == "Runtime 06 F8 RuntimePluginDescriptor public constructor retired" {
        "2026-06-22"
    } else if slice == "Runtime 07 Performance hotpath Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 07 scene/EventBus poison-safe locks" {
        "2026-06-22"
    } else if slice == "Runtime 07 render submit source-extract sharing" {
        "2026-06-22"
    } else if slice == "Runtime 07 render camera-loop descriptor submissions" {
        "2026-06-22"
    } else if slice == "Runtime 07 render camera-loop frame terminal move" {
        "2026-06-22"
    } else if slice == "Runtime 07 render submit feedback sideband owned merge" {
        "2026-06-22"
    } else if slice == "Runtime 07 render prepared sideband frame owner move" {
        "2026-06-22"
    } else if slice == "Runtime 07 render direct runtime-frame streaming camera loop" {
        "2026-06-22"
    } else if slice == "Runtime 07 render generated camera-loop shared extract" {
        "2026-06-22"
    } else if slice == "Runtime 07 render shared effective extract frame source" {
        "2026-06-22"
    } else if slice == "Runtime 07 render direct runtime-frame shared context extract" {
        "2026-06-22"
    } else if slice == "Runtime 07 render VG debug overlay frame override" {
        "2026-06-22"
    } else if slice == "Runtime 07 render direct runtime-frame trace export" {
        "2026-06-22"
    } else if slice == "Runtime 07 render submit effective extract projection" {
        "2026-06-22"
    } else if slice == "Runtime 08 F5 world typed mutation errors" {
        "2026-06-22"
    } else if slice == "Runtime 08 F5 dynamic component typed errors" {
        "2026-06-22"
    } else if slice == "Runtime 02 F6 core resource registry typed errors" {
        "2026-06-22"
    } else if slice == "Runtime 07 Performance hotpath inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 12 input validation window recheck" {
        "2026-06-20"
    } else if slice == "Runtime 12 Input stack current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 12 Input stack inventory split"
        || slice == "Runtime 12 Input stack Markdown renderer split"
        || slice == "Runtime 12 input boundary grouped manager import guard repair"
    {
        "2026-06-21"
    } else if slice == "Runtime 14 module family current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 14 module family markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 02 core/root/generated current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 02 core/root/generated Markdown renderer split"
        || slice == "Runtime 02 generated-code Markdown renderer split"
        || slice == "Runtime 02 root-surface Markdown renderer split"
    {
        "2026-06-21"
    } else if slice == "Runtime 03 Schedule/frame-loop current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 03 Schedule/frame-loop inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 03 Schedule/frame-loop markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 13 Script binding current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 13 script binding Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 08 ECS 数据面 current audit recheck" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS source/test inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 08 ECS anchor inventory split" {
        "2026-06-21"
    } else if slice == "Runtime 08 ECS markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 08 QueryState Markdown renderer split" {
        "2026-06-21"
    } else if slice == "Runtime 08 ECS event owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS message owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS resource store owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS resource identity owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS component identity owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS entity identity owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS archetype owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS component storage owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS component storage private re-export cleanup" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS observer owner folder split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS commands facade owner split" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS command Cargo 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS entity Cargo 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS data owner-tree guard" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS change detection owner-tree guard" {
        "2026-06-20"
    } else if slice == "Runtime 08 ECS root leaf owner guard" {
        "2026-06-20"
    } else if slice == "Runtime 08 ecs_events_messages Cargo 验证窗口探测" {
        "2026-06-20"
    } else if slice == "Runtime 11 JobSystem 行为测试锚审计同步" {
        "2026-06-17"
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
        "2026-06-20"
    } else if slice == "Runtime 04 Asset pipeline inventory split"
        || slice == "Runtime 04 Asset pipeline Markdown renderer split"
    {
        "2026-06-21"
    } else if slice == "Runtime 04 F7 asset artifact/importer typed errors"
        || slice == "Runtime 04 F8 texture import settings apply API"
    {
        "2026-06-22"
    } else {
        "2026-06-14"
    }
}
