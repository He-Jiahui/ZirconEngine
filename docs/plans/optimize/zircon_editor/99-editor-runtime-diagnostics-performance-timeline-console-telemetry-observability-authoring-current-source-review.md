---
title: Editor Runtime Diagnostics / Performance Timeline / Console / Telemetry / Observability Authoring 当前源码复审
category: zircon_editor
report_id: Editor99
review_date: 2026-08-26
baseline_head: 38c0e7f5d48189ac2637ed010e452b19c32f459d
verification_head: 38c0e7f5d48189ac2637ed010e452b19c32f459d
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.zui
  - zircon_editor/src/core/gateway/session/profile.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/performance_timeline_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders
  - zircon_editor/src/ui/retained_host/app/profiling
  - zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/workbench_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_host/src/foreign_output/item_count.rs
  - zircon_plugins/runtime_diagnostics
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/99zg-runtime-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Public/TraceServices/Model/AnalysisSession.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Public/TraceServices/AnalysisService.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Public/Insights/IInsightsManager.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Public/Insights/ITimingViewSession.h
  - dev/godot/editor/debugger/editor_debugger_plugin.h
  - dev/godot/editor/debugger/editor_debugger_node.h
  - dev/godot/editor/debugger/editor_performance_profiler.h
  - dev/godot/editor/debugger/editor_profiler.h
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Fyrox/editor/src/stats.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/IDebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugFrameTiming.cs
---

# Editor99 · Runtime Diagnostics / Performance Timeline / Console / Telemetry 当前源码复审

## 1. 结论

本轮不是“没有诊断代码”的结论。Runtime 已有 62 个 diagnostics/profiling 文件，包含有界 series history、当前值快照、render/physics/animation provider、CPU frame/span/counter recorder、热点分析、Perfetto/JSON 导出、retention evidence 与局部 capture epoch。Editor 已有真实 `editor.runtime_diagnostics`、`editor.performance_timeline` descriptor、可见性门控、pane payload、native timeline 行裁剪和 profile 控制。动态 ABI 也已经能返回子 Runtime 的 `RuntimeDiagnosticsSnapshot`。

问题是这些散件仍没有形成工程级 observation product。四个 Workbench 扩展页面仍是 16 个文件、5,562 行的静态 fixture：Session、Frame、Actors、DAU、Crash Rate、事件数和控制反馈来自 ZUI/回调常量。真实 Runtime Diagnostics 面板仍先读 Editor host `EditorManager::runtime_diagnostics()`，只对 child Runtime 发 `ProfileControlCommand::Snapshot`，不消费 child 的 `runtime_diagnostics`，也不把 store series 变成 pane model。Performance Timeline producer 仍把 frames、spans、hotspots 各自截为最近 12 行，counter 只进入 summary 文本；转换器虽然能从 10,000 个逻辑行只 materialize 可见行，但它没有改变上游 12 行数据合同。

跨进程 profile 仍不可信。Editor 和 Runtime 各自用本地 `Instant` origin，`ProfileSnapshot` 没有 source/process/clock/base time/generation；merge 只偏移 span id、拼接 vectors、OR 状态并合并 retention。capture epoch 目前只保护 realtime IBL 异步回报，不参与双进程 capture coordinator、ABI 或 snapshot identity。presentation 可见时仍同步查询 render/physics/animation、构造 full diagnostic history、clone profile rings 并等待 Runtime FFI。`current_snapshot()`、authority store 写回、VG availability 查询和 ring capacity 修复只是局部 observer-effect 降低，不能替代后台 collector/cache。

`DiagnosticStore` 仍可由任意字符串路径隐式创建；总 series、path、unit、tag、owner/cardinality 和 finite-value budget 缺失。Telemetry 生产依赖、provider、schema、consent、redaction、tenant、retention、ingest 全部为 0；Dashboard 仍伪造运营事实。Runtime diagnostics plugin 继续注册 builtin view id，并引用不存在的 `plugins://runtime_diagnostics/editor/authoring.zui`。因此本轮重判：6 个 P0 为 Open 5 / Partial 1；60 个 P1 为 Open 54 / Partial 5 / Closed 1；12 个 P2 为 Open 11 / Partial 1；32 个资格门为 Fail 28 / Partial 4 / Pass 0。旧 Editor25 只在 currentness 上被本报告取代，架构目标和 ID 保持不变。

## 2. 当前源码范围与证据

### 2.1 冻结选择集

| 选择集 | 文件 / 行 / 非空行 / bytes / test / ignored | fingerprint |
|---|---:|---:|---:|---:|---|
| Workbench 静态 diagnostics surface、binding、navigation、feedback | 16 / 5,562 / 5,265 / 288,169 / 2 / 0 | `233d3f68fb9530b597cbc2695d726118ff88afca6a0f33c4fc186e94ea1a0187` |
| Editor real observability product、gateway、pane、visibility、host lifecycle | 71 / 14,108 / 13,337 / 524,249 / 61 / 3 | `f2c0183fd62801797a5ac7589923dd25e02b1bbf3328d47ee0ec0a4a532dd297` |
| Runtime diagnostics、profiling、dynamic ABI 与 output budget | 73 / 15,161 / 14,361 / 500,040 / 94 / 2 | `4b3bf6baee9dba1efc9a2ccf2e12efdc2f61f5063afe68b30e05899e69bb35f1` |
| `runtime_diagnostics` plugin package | 9 / 436 / 395 / 16,128 / 4 / 0 | `4f68b585158eb145d169ae6b5778cfde2a2d066c2595b7e06509663265005aaf` |
| focused tests | 15 / 4,684 / 4,434 / 168,787 / 37 / 0 | `9a2b452c6179192625346a1a001d8da443259dd11e0dff98b5aa3c07dd5abb31` |
| selected union | 180 / 39,014 / 36,868 / 1,460,580 / 198 / 5 | `508b6b782e4dea09fae5c71cccf2b7c940987ff6cd4e89d2ee586f3dece32315` |

统计脚本按 UTF-8 物理行、非空行、bytes 和 Rust test attributes 计数；fingerprint 是按相对路径排序的 `path<TAB>file_sha256` 清单 SHA-256。选择集包含当前在途 Runtime graph split、profiling epoch、recorder retention、ZUI button state 变更；本报告不回退也不吸收这些变更。五套参考源码共 17 文件、11,021 行、406,228 bytes，fingerprint `414d48196c96d70b0959e36a73833ce19ed4c0a3655f2dd37f6ba6cc0d7b9fbf`。

### 2.2 动态证据边界

本轮 review-only，不运行 Cargo、Editor 窗口、长时 capture、跨进程时钟校准、650 指标压力、断线重连、Perfetto 回放、Telemetry backend 或隐私测试。198 个 test attributes 是源码 inventory，不是通过数。上一轮 Editor library test 仍受既有编译阻断，不能把静态测试合同写成动态通过。实施前必须重取选择集、fingerprint、ABI 和工作树在途文件。

### 2.3 责任边界

- Editor99 拥有 observability product、observation session、跨进程 source/clock/query、timeline model/presentation、capture UI 状态和 Workbench truthfulness。
- Editor11 拥有 Console journal、routing、retention、export；Runtime132 拥有 process log router/sink/crash；Runtime107 拥有 console/CVar/remote command；Runtime03 拥有底层 DiagnosticStore/profile recorder/export；Editor09/07 拥有 jobs 与 Play lifecycle。
- 本报告不得在 Editor 侧复制一套日志、recorder、remote transport 或 Telemetry authority。

## 3. 必须保留的真实基础

1. 保留 runtime `DiagnosticStore` 的 per-series history、current-only snapshot、static metadata fast path 和 render/physics/animation availability；修复 descriptor/cardinality/clock，不推倒 provider。
2. 保留 child Runtime ABI 已返回的 project、scene、device、input、reload、series、profile payload；缺口是 Editor consumer、source identity 和 freshness，而不是重新发明 DTO。
3. 保留 profiler frame/span/counter ring、hotspot、Perfetto/JSON、retention、finite budget normalization 和 capture epoch 的局部正确性；统一到 session/generation 后再扩展。
4. 保留 pane visibility gate、native timeline visible-row conversion、UI debug reflector 和真实 Editor Console journal；它们是 cache/query 终端，不是 observation authority。
5. 保留插件 descriptor/extension 机制，但 builtin view 只能有一个 owner，plugin 必须解析真实资源并声明 capability/maturity。

## 4. 当前断路

```text
Workbench fixtures ----> local control mutation / fixed feedback (no provider)

Editor presentation --> host Runtime diagnostics + full clone + child Profile snapshot FFI
                      \-> child RuntimeDiagnosticsSnapshot is returned but unconsumed

Editor recorder ---- local Instant/origin ----> merge by span-id offset + vector append
Runtime recorder --- local Instant/origin ----> same fake session/timeline

DiagnosticStore -- arbitrary path -> BTreeMap -> full history clone under mutex
Telemetry dashboard ----------------------------------------> no provider / no governance
runtime_diagnostics plugin -> builtin view id + missing authoring.zui
```

## 5. P0 重判

| ID | 当前状态 | 当前证据与必须重构 |
|---|---|---|
| P0-1 | **Open** | 4 个 ZUI workspace 共 108 个 `[nodes.*]` block、76 条 route；固定 session/metrics/feedback，必须显式 Demo/Unavailable 或接入真实 provider。 |
| P0-2 | **Open** | `runtime_diagnostics_with_profile()` 只消费 host `EditorManager` 和 child profile snapshot；child Runtime diagnostics ABI 无 Editor consumer，必须建立 source-qualified session 与真实 series projection。 |
| P0-3 | **Open** | `snapshot_merge.rs` 仍无 process/source/clock/base/generation，只做 span offset、OR、append、session 拼接；必须停止物理伪合并，改多 track。 |
| P0-4 | **Partial** | authority store 写回、current-only periodic snapshot、VG availability query、ring capacity 和 finite budget 等减少部分成本；但 visible presentation 仍同步 query/full clone/profile FFI，必须迁移后台 collector/cache。 |
| P0-5 | **Open** | `BTreeMap<DiagnosticPath, DiagnosticSeries>` 仍允许 arbitrary path，只有 history 64；总 series、metadata/cardinality、owner lease、finite value 未受控。 |
| P0-6 | **Open** | Cargo manifest 中无 telemetry/analytics/OpenTelemetry 依赖，未发现 provider/schema/consent/redaction/tenant/retention/ingest；固定 DAU、Crash Rate 必须移除。 |

## 6. P1 重判：Observation Session、Metric、Remote、Timeline、Telemetry

### 6.1 Observation Session、Clock 与 Transport

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-1 | Open | `ProfileSnapshot` 仍无 stable source/process/runtime generation；添加 `ObservationSourceId`、PID/process UUID、target role 和 project/world identity。 |
| P1-2 | Open | Editor/Runtime 各自 `Instant::now()` origin，无 clock domain/base/calibration/error；未校准源不得排序或聚合。 |
| P1-3 | Open | 只有 visibility、dynamic session 和 recorder active，缺少 connect/restart/disconnect/terminate observation lifecycle；建立 generation-fenced registry。 |
| P1-4 | Open | capture/query/export/reset 仍同步命令、状态字符串、无 request/deadline/idempotency/terminal receipt；改 typed coordinator protocol。 |
| P1-5 | Open | Dynamic ABI 没有 source capability negotiation；UI 不应默认宣称 series/watch/GPU/export 都可用。 |
| P1-6 | Open | Runtime snapshot 无 collected_at、source、freshness、age、stale/disconnected/error；pane payload 必须显示这些字段。 |
| P1-7 | Open | `ProfileControlResponse.status/message` 与 action status 仍自由字符串；稳定 error code/source/retryability 必须进入公共接口。 |
| P1-8 | Open | render/physics/animation/store/profile 仍在不同时间点采集，snapshot 无 provider generation/consistency boundary。 |
| P1-9 | Open | 传输仍完整 JSON request/response；增加 bounded page/delta/full recovery、sequence、schema/version、bytes/depth budget。 |
| P1-10 | Open | child FFI error/unavailable 在 diagnostics helper 中回退 host snapshot，UI 看不到 disconnected/stale/error；错误必须成为可查询事实。 |

### 6.2 Metric Registry、Collector 与 Snapshot Store

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-11 | Open | `DiagnosticStore::record` 可隐式建 series；先注册 descriptor，未知/冲突写入拒绝并计数。 |
| P1-12 | Open | string path 同时承担 identity、display 和 storage key；引入 dense MetricId 与稳定 display path。 |
| P1-13 | Open | unit 是可覆盖 String；descriptor 固定 kind/unit/version，变更需新 generation。 |
| P1-14 | Open | tags 永久 union，虽有 HashSet 投影/本地 perf evidence，仍无 bounded label/eviction。 |
| P1-15 | Open | 无 owner lease/deregister；plugin unload/world close/session end 可能留下 provider state。 |
| P1-16 | Open | history 64 不是全局 cardinality budget；series/path/tag/bytes/owner 总额需 admission。 |
| P1-17 | Open | path/tag/unit 没有长度和 encoded bytes budget；动态输入不能成为无界 key。 |
| P1-18 | Open | counter hotspot 过滤非 finite 只保护分析输入，DiagnosticStore record 仍接收 NaN/Inf。 |
| P1-19 | Open | EMA 仍按样本固定 0.9/0.1，measurement 没有 monotonic time；改 time-constant/descriptor aggregation。 |
| P1-20 | Open | min/max 仍为 process/lifetime extrema，history eviction 不重算 window 统计。 |
| P1-21 | Partial | recorder retention 给出 written/overwritten/retained/oldest/newest，改善完整性；measurement 仍无 source/sequence/time/generation，无法构造可靠窗口。 |
| P1-22 | Partial | `current_snapshot()` 分离了周期日志的 history/tags clone，但 full `snapshot()` 仍深复制，UI path 仍可请求完整 store；改 immutable generation/page。 |
| P1-23 | Partial | collector 现在在 `update_diagnostic_store` 内维护权威 store，减少了额外临时 store clone；`CoreHandle` 仍在 diagnostics mutex 下 clone/snapshot，深复制仍在热路径。 |
| P1-24 | **Closed** | `collect_runtime_diagnostics` 已通过 `update_diagnostic_store` 写回 render/physics/animation，再 snapshot；旧结论“派生 history 每次从空 clone 开始”不再适用于当前源码。后续仍需 source/time semantics，但本 ID 的具体 bug 已关闭。 |
| P1-25 | Open | provider 没有统一 cadence/cost/admission/last-success/collection latency；cheap VG availability 不是 health contract。 |

### 6.3 Remote Runtime、Console 与 Timeline

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-26 | Open | builtin Runtime Diagnostics payload 仍只有 summary/render/physics/animation/detail strings，没有 series/history/source/freshness。 |
| P1-27 | Open | `dynamic_api/session/diagnostics.rs` 生成 child payload，但 Editor 只请求 profile；建立 typed consumer。 |
| P1-28 | Open | 无多 Runtime source 选择、对比或 host/child/server role；Observation Session registry 必须可枚举。 |
| P1-29 | Open | world query/watch 未接 Diagnostics provider，不能关联 selected asset/node/entity 与 metric generation。 |
| P1-30 | Open | watch 没有 typed address/schema/owner generation；禁止把字符串 query 直接嵌入 pane。 |
| P1-31 | Open | 无 rate/history/trigger/budget 的 watch subscription；必须走后台 bounded stream。 |
| P1-32 | Open | Events tab 没有真实 event provider；固定事件数不可保留。 |
| P1-33 | Open | remote inspect/edit 权限、provenance、audit 不在本产品表达；只读/写能力必须 capability-gated。 |
| P1-34 | Open | break/step/stack/callsite 没有 debugger provider；不要用 profile span 文本冒充 callstack。 |
| P1-35 | Open | Workbench Console Diagnostics 仍复制 Editor11 Console journal；改为 journal query projection，不能再维护 fixture。 |
| P1-36 | Partial | producer 仍 frame/span/hotspot 各取 12 行，产品不是连续 timeline；converter 已有 10K logical-row visible clipping，只保留为后续 virtualization foundation。 |
| P1-37 | Open | 无 zoom/pan/range/marker/selection 语义；固定列表不能推导时间轴交互。 |
| P1-38 | Open | stream/category/name 仍只是字符串，缺 stable track/thread/process/queue identity。 |
| P1-39 | Open | counters 只影响 summary/has_samples，未形成 counter track/row/graph。 |
| P1-40 | Open | CPU/GPU timestamp 无 correlation、queue、calibration/error；不得在同一 track 直接比较。 |
| P1-41 | Open | frame 只有 recorder-local index；需要 source/generation/global sequence。 |
| P1-42 | Open | parent ID 不含 async flow/task/thread relation；analysis provider 需 typed edges。 |
| P1-43 | Open | hotspot 按 stream/category/name/path 字符串聚合，未带 source/callsite identity。 |
| P1-44 | Open | merge 无 runtime budget/source config，retention 只被 append，不能解释每个 recorder 的 effective budget。 |
| P1-45 | Open | producer reverse/take 12 按 append 顺序展示，非基于统一 timestamp/range。 |
| P1-46 | Open | dispatch 先同步控制 Editor recorder，再同步 child FFI，无 prepare/ack/timeout/cancel/rollback。 |
| P1-47 | Open | reset/stop 仍可能先改变 host recorder 再 child 失败；capture epoch 只在 runtime IBL async path 有效。 |
| P1-48 | Open | export 仍 `create_dir_all + fs::write` 固定文件名，无 manifest/hash/atomic publish/source isolation；Editor/Runtime 默认目录可碰撞。 |
| P1-49 | Open | pane presentation/command path 同步 snapshot、分析、FFI；必须后台 job + immutable cache。 |
| P1-50 | Open | 没有 TraceStore + typed analysis provider 层，pane builder 直接从 ProfileSnapshot 拼字符串。 |

### 6.4 Telemetry、Plugin 与资格

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-51 | Open | 无 telemetry event schema registry/provider；Dashboard 不得显示静态 DAU。 |
| P1-52 | Open | 无 consent/classification/redaction；runtime/profile 本地数据不可默认外发。 |
| P1-53 | Open | 无 endpoint/auth/tenant/environment；仅有 UI 字符串不能成为服务。 |
| P1-54 | Open | 无 ingest/offline queue/delivery receipt；不要把“Filter Telemetry”反馈当作投递。 |
| P1-55 | Open | 无 retention/deletion/aggregation policy；运营数据和诊断数据生命周期必须分离。 |
| P1-56 | Open | Crash Rate 没有 crash evidence source；固定 0.18% 必须删除或标 unavailable。 |
| P1-57 | Open | plugin 注册 `plugins://runtime_diagnostics/editor/authoring.zui`，实际资源不存在；先修资源解析/owner contract。 |
| P1-58 | Open | plugin 与 builtin 都使用 `editor.runtime_diagnostics`；必须单一 owner，plugin 只能 provider/extension。 |
| P1-59 | Open | plugin catalog/descriptor/asset resolution 语义仍矛盾，测试只断言字符串；需要 package load report。 |
| P1-60 | Partial | retention、current snapshot、capture epoch、visible-row conversion、finite budget 和局部 source-contract tests 提高基础质量；仍没有双进程、断线、规模、artifact、privacy 的闭环资格。 |

## 7. P2 重判

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P2-1 | Open | Diagnostics、Debug Observatory、Runtime Diagnostics 仍缺统一 provider/subject/terminology。 |
| P2-2 | Open | internal `RuntimeDiagnosticsSnapshot` 与 ABI payload 仍分离，需公共 versioned projection。 |
| P2-3 | Open | status/message/feedback 仍大量 String 拼接；改 typed outcome + localization/presentation。 |
| P2-4 | Open | `MAX_TIMELINE_ROWS=12` 仍是 producer 固定策略，converter clipping 不能取消它。 |
| P2-5 | Open | artifact/output path 直接落为 UI String，需 artifact id/manifest/source identity。 |
| P2-6 | Open | diagnostic rows 没有 source-qualified jump/freshness/subject target。 |
| P2-7 | Open | timeline source/filter/track/range preference 没有持久化合同。 |
| P2-8 | Open | accessibility、keyboard range/zoom/selection 未定义。 |
| P2-9 | Open | live/current、captured、stale、partial 的视觉语义未分离。 |
| P2-10 | Open | `snapshot_merge` 仍用 saturating span-id offset；必须删除伪合并，而非换更大整数。 |
| P2-11 | Open | plugin maturity/capability 没有传播到 view availability。 |
| P2-12 | Partial | 本报告补充了五套参考边界、owner 分工和 currentness recheck 纪律；源码仍未满足 runtime/editor reference boundary 文档要求。 |

## 8. 参考引擎对照

| 参考 | 当前源码可验证机制 | Zircon 应吸收 |
|---|---|---|
| Unreal TraceServices / TraceInsights | `IAnalysisSession` 有 Stop/Wait、duration、base date/time、metadata、BeginRead/BeginEdit 和 provider registry；Timing View 有 tracks、time marker、selection、relations。 | observation session、provider ownership、读写 scope、base time、analysis completion、stable track/marker/selection；不要复制 Slate。 |
| Godot debugger | debugger node 管理 session、active/breaked/disconnected 状态、message channel；performance/profiler 单独控制 capture 与历史。 | 多 session lifecycle、断线/暂停状态、profiler capability 和 UI 状态分离；不要将一个下拉字符串当 session authority。 |
| Bevy diagnostics/remote | DiagnosticPath 先约束/注册，measurement 带 `Instant`，history、EMA、enabled 和 bounded length 明确；Remote 独立 plugin、JSON-RPC method/result/error/OpenRPC discovery，HTTP transport 另装。 | registry/time-aware smoothing/disabled fast path；transport、method schema、error 与 UI 解耦。Bevy HTTP 默认不等于 Zircon 安全策略。 |
| Fyrox | engine 持有 PerformanceStatistics、elapsed time、task pool、scene graph；stats surface 是窄而真实的运行时下限。 | domain ownership、elapsed/frame stats、scene identity；不能把简单文本 stats 当 Timeline 上限。 |
| Unity Graphics | `DebugManager` 注册/取消 debug data，panel 可 create/replace/remove，dirty refresh 与 reset 明确；`DebugFrameTiming` 是有限 frame timing history。 | provider/panel lifecycle、reset/dirty、bounded timing history、runtime/editor display capability；不能从 Graphics 包推断完整 Profiler/Analytics。 |

## 9. 分层重构路线

1. **M0 Truthfulness**：四个静态 Workbench surface 改为真实 provider 或显式 Unavailable；删除固定 DAU、Crash Rate、Session/Frame/Actors 文案和 queued 假 receipt；修复 plugin resource/owner 冲突。
2. **M1 Observation Session**：建立 Editor-owned registry，source/process/runtime generation、project/world/target role、capability、connection/freshness/last error 进入统一 projection。
3. **M2 Clock/Transport**：为 ABI/profile/diagnostic DTO 增加 source、clock domain/base/calibration、sequence、schema、request/deadline/idempotency；page/delta/full recovery 有 bounded budget。
4. **M3 Metric Cache**：MetricDescriptor registry、owner lease、cardinality/bytes/finite admission；Runtime fixed cadence 发布 immutable generations，Editor pane 只读 cache。
5. **M4 Runtime Diagnostics**：Editor 真实消费 child `RuntimeDiagnosticsSnapshot`，显示 series/history/current/store source、collected_at、age、partial/stale/error；host 与 child 不再互相冒充。
6. **M5 Trace Analysis**：source-qualified TraceStore 与 providers；CPU thread/task、GPU queue、counter、frame、async flow、journal relation 通过 stable track/edge 输出。
7. **M6 Capture/Artifact**：prepare/start/stop/seal/finalize coordinator；per-source receipts、partial/degraded、epoch/generation、staging/manifest/hash/atomic publish/source-isolated paths。
8. **M7 Timeline**：取消 producer 12-row cap，按 range/query 分页、zoom/pan/marker/selection、counter tracks、virtualized rendering 和 million-event budget 验收。
9. **M8 Console/Plugin convergence**：Timeline/Runtime/Console 都经各自 canonical owner；Console 只查询 Editor11 journal，plugin provider 必须带 capability/maturity/lease。
10. **M9 Telemetry**：仅在独立 provider、consent、redaction、auth、tenant、retention、deletion、offline delivery receipt 与 crash evidence gate 全部满足后开放。

## 10. 验收门禁重判

| Gate | 状态 | 说明 |
|---|---|---|
| G01/G02/G03 | Fail | fixtures、duplicate owner、missing plugin resource 均未关闭。 |
| G04-G14 | Fail | source/clock/session/request/capability/snapshot/registry/cardinality/value/lease 均无完整合同。 |
| G15 | Partial | per-series history、ring limits、output byte/item budgets 和 current-only snapshot 存在；650+ end-to-end collection budget 未证明。 |
| G16-G20 | Fail | presentation 仍 sync IO/query/clone/FFI；freshness、runtime source、watch 未闭环。 |
| G21 | Partial | profile retention 有 overwrite/oldest/newest，仍无统一 event sequence/drop stream。 |
| G22 | Fail | Workbench Console Diagnostics 未收敛到 Editor11 journal。 |
| G23 | Partial | conversion 对 10K logical rows 做 visible clipping，producer 仍 12 rows，未有 zoom/pan/range/million-event evidence。 |
| G24-G31 | Fail | stable tracks/correlation/capture transaction/reset/artifact/telemetry/privacy 均未通过。 |
| G32 | Partial | 局部 source-contract、retention、epoch、finite/ring tests 存在；断线、双进程、规模、artifact collision、missing resource、privacy 未执行。 |

## 11. 禁止的临时修补与本轮边界

- 禁止只替换 fixture 数字、把 `MAX_TIMELINE_ROWS` 提高、继续给 span id 做偏移，或把 child FFI 错误吞掉后显示 Live。
- 禁止在 presentation 中增加更多 render/GPU query、full clone、同步 FFI；必须先引入后台 cache 与 typed receipt。
- 禁止把 arbitrary metric path、entity/request/asset/user input 拼成长期 series；禁止在没有 Telemetry governance 时接 endpoint。
- 禁止复制日志、recorder、remote transport、plugin ZUI 来绕过 Editor11/Runtime03/Runtime132 的 canonical owners。
- 本轮仅完成 review 与 currentness 文档，未修改生产代码，未执行 Cargo 或动态产品验证。实施前应重取 baseline/fingerprint 并重新检查共享工作树在途文件。
