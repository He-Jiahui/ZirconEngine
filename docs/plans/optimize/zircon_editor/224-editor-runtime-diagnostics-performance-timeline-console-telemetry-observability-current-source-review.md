---
title: Editor Runtime Diagnostics / Performance Timeline / Console / Telemetry / Observability 当前源码复核
category: zircon_editor
report_id: Editor224
review_date: 2026-08-29
baseline_head: b2e76ff33cc298ad76f7b801a1d06d1e2faa046d
verification_head: b2e76ff33cc298ad76f7b801a1d06d1e2faa046d
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor99
refreshes_currentness_of:
  - docs/plans/optimize/zircon_editor/99-editor-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.zui
  - zircon_editor/src/core/gateway/session/profile.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/performance_timeline.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/profiling
  - zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/performance_timeline.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_plugins/runtime_diagnostics
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

# Editor224 · Runtime Diagnostics / Performance Timeline / Console / Telemetry 当前源码复核

## 1. 结论

当前代码不是“完全没有诊断能力”。Runtime 已有 `DiagnosticStore`、render/physics/animation provider、frame/span/counter recorder、hotspot 分析、retention 计数、JSON/Perfetto 导出和 typed export error；dynamic session 也会生成包含 project、scene、device、input、diagnostic series、scene reload 与 profile 的 `RuntimeDiagnosticsSnapshot`。Editor 已有两个真实 builtin view、可见性门控、pane payload、host contract、timeline 可见行裁剪、capture/start/stop/export/reset action 以及 UI debug reflector。

但这些局部底座尚未组成工程级 observability product。四张可导航 Workbench workspace 仍把 Session、Frame、Actors、Events、DAU、Crash Rate、延迟和反馈写死在 ZUI/Rust 常量里；`capture`、`export`、`run query` 只返回 queued 文本。真实 pane 只从 `EditorManager::runtime_diagnostics()` 取得 host snapshot，再向 child Runtime 发 `ProfileControlCommand::Snapshot` 读取 profile；child 已返回的 `response.runtime_diagnostics` 没有 Editor consumer。Runtime Diagnostics pane 主要把 render/physics/animation 状态格式化成字符串，没有 series/history/source/freshness model。

跨 recorder 的 profile 合并仍不可信。Editor 与 Runtime 各自使用本地 `Instant` origin，公共 `ProfileSnapshot` 没有 source/process/thread/clock/base/generation；`merge_profile_snapshot()` 仅做最大 span id 偏移、session 字符串拼接、状态 OR 和 vector append。Performance Timeline producer 对 frames、spans、hotspots 分别 `take(12)`，counter 没有 track；转换层的可见行裁剪只是渲染底座，不能把三组短列表变成可缩放时间轴。

Metric 和 Telemetry authority 也未成立。`DiagnosticStore::record()` 可以用任意字符串隐式创建 series，unit 可覆盖、tags 永久 union、值可为 NaN/Inf，只有单 series history=64 而没有全局 cardinality/bytes/owner lease。生产 Runtime/Interface/Plugin 中未发现 `TelemetryProvider`、`TelemetryEvent`、`TelemetrySchema` 或 `TelemetryConsent` 合同。Profile export 仍对 caller `output_root` 执行 `create_dir_all` 和多次 `fs::write`，没有 staging/manifest/hash/atomic publish。`runtime_diagnostics` plugin 还与 builtin 共用 `editor.runtime_diagnostics`，并引用实际不存在的 `plugins://runtime_diagnostics/editor/authoring.zui`。

因此本轮保持 Editor99 的 canonical finding 身份并按当前树重判：P0 为 **5 Open / 1 Partial**，P1 为 **54 Open / 5 Partial / 1 Closed**，P2 为 **11 Open / 1 Partial**；32 项资格门为 **28 Fail / 4 Partial / 0 Pass**。本报告不新增唯一 finding，不改变全局 P0/P1/P2 总数。

## 2. 当前源码范围

### 2.1 选择集

| 选择集 | files / lines / nonempty / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Workbench diagnostics workspace、HUD/index、navigation/feedback | **8 / 2,244 / 1,992 / 128,249 / 0 / 0** | `53fa4fa75886ea8a3b19f419243dddf24f7e05c26e1e55780874e00033777af0` |
| Editor gateway、descriptor、pane、profiling、visibility、reflector | **34 / 6,070 / 5,603 / 216,573 / 48 / 5** | `8d484f19f420e6bbec191ab2cd75ed2ddcd2a1fccc90b6d15a3db798f51af9a7` |
| Runtime diagnostics/profile、dynamic ABI、Interface/Host output contract | **88 / 15,248 / 14,446 / 502,465 / 103 / 5** | `7f27ff147d0bf8fd01e01ce4f5d116cd5619eac7d4572cf07b5b33604c85d6a2` |
| `runtime_diagnostics` plugin | **9 / 436 / 395 / 16,128 / 4 / 0** | `cb829cb8247c46a3072eb768ea6dfd3073fb32b2acf7c132d477aa375be1a4e7` |
| focused Editor host/pane/template tests | **17 / 5,090 / 4,806 / 189,358 / 49 / 0** | `c004c0c3230deafc9b368c776590a9bd7e864aa9570284d039b5225aa15fc5be` |
| selected union | **156 / 29,088 / 27,242 / 1,052,773 / 204 / 10** | `f0caa552d45dfb797bca506238b6479cb98f415c3427a4b83bbd89653fdeffee` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics reference selection | **17 / 11,021 / n/a / 406,228 / n/a / n/a** | `0ce2b476d82e5af7933cccca8cb89ca51ed4cff958e22307c13a7034b2e920d0` |

统计按 UTF-8 物理行、非空行、bytes、Rust test attributes 和 ignored attributes 计算；fingerprint 是按相对路径排序的 `path<TAB>file_sha256` 清单 SHA-256。选择集刻意只纳入两个诊断 pane 的直接 conversion 文件，不把 238 个共享 pane conversion 文件伪装为本产品代码；共享布局框架由 Editor01 owner 负责。本轮保留工作树中既有修改与 untracked split，不回退或吸收它们。

### 2.2 动态证据边界

本轮是 review-only，没有运行 Cargo、Editor、Runtime DLL、真实多进程 capture、断线恢复、百万事件 timeline、Perfetto 回放、artifact collision、Telemetry backend、privacy、fault、scale、soak 或动态 benchmark。204 个 test attributes 是源码 inventory，不是通过数。结构检索发生超时时改用更窄路径和 `git grep` 继续核对，不把超时记为 Pass，也不等待协调器。

### 2.3 Owner 边界

- Editor99/Editor224：observation session、跨进程 source/clock/query、timeline presentation、capture UI 状态和 Workbench truthfulness。
- Runtime03/Runtime156：底层 `DiagnosticStore`、profile recorder、export 与 config authority。
- Editor11：Console journal/query/export；Runtime132：process log router/sink/crash；Runtime107：console/CVar/remote command。
- Editor09/Editor07：background jobs 与 Play session lifecycle；Plugin owner只提供 provider/extension，不复制 builtin view authority。

## 3. 必须保留的真实底座

1. `runtime_diagnostics_response()` 已生产 child project/scene/device/input/reload/series/profile payload；重构目标是 source-qualified consumer，不是再造一套 DTO。
2. 保留 per-series bounded history、`current_snapshot()`、static metadata fast path、recorder ring 和 retention oldest/newest/overwritten 证据。
3. 保留 Runtime Diagnostics / Performance Timeline descriptor、visibility gate、pane payload/host contract、visible-row clipping 与 UI debug reflector。
4. 保留 profile output byte/item budget、session basename sanitization、typed export error 与 Perfetto compatibility；补齐 artifact transaction，而非删除导出能力。
5. 保留插件 descriptor/capability/distribution 基础，但 view、resource、maturity、lease 和 package load receipt 必须一致。

## 4. 当前实际数据流

```text
Workbench diagnostics workspaces
    -> navigation spec
    -> fixed local feedback / fixed table facts
    -> no observation provider

visible diagnostics pane
    -> EditorManager host RuntimeDiagnosticsSnapshot
    -> full store/profile clone
    -> child ProfileControlCommand::Snapshot FFI
    -> append editor/runtime profile vectors
    -> format strings / 12-row lists

child ProfileControlCommand::RuntimeDiagnosticsSnapshot
    -> RuntimeDiagnosticsSnapshot with real diagnostic_series
    -> response.runtime_diagnostics
    -X no Editor consumer

Telemetry Dashboard
    -> fixed DAU / Crash Rate / 2.4M events / 120 ms
    -X no schema / consent / provider / ingest / receipt
```

## 5. P0 重判

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| P0-1 | **Open** | 四张 production ZUI 仍固定 `Session_12_10`、`Capture_1234`、`Session_Player_01`、`DAU 128K`、`Crash Rate 0.18%`；回调继续固定 queued/selected 文本。必须接真实 provider 或明确标成 Demo/Unavailable。 |
| P0-2 | **Open** | `runtime_diagnostics_with_profile()` 只消费 host diagnostics 与 child `response.snapshot`；child `response.runtime_diagnostics` 在 Editor/Plugin 中零消费。必须建立 source-qualified observation registry 与真实 series projection。 |
| P0-3 | **Open** | `ProfileSnapshot` 无 source/process/clock/base/generation，`snapshot_merge.rs` 只做 span offset、OR、append、session 拼接。必须删除物理伪合并，改为每 source 独立 track/query。 |
| P0-4 | **Partial** | visibility gate、current snapshot、ring retention、有限值 budget normalize 与 visible-row conversion 减少局部成本；但 presentation 仍同步 query/full clone/analysis/FFI。必须迁移后台 collector + immutable cache。 |
| P0-5 | **Open** | `BTreeMap<DiagnosticPath, DiagnosticSeries>` 仍允许任意 path 隐式建 series，只有 per-series history cap；缺 descriptor、全局 cardinality/bytes、owner lease、finite admission。 |
| P0-6 | **Open** | 生产 Runtime/Interface/Plugin 对 telemetry provider/schema/consent 为零命中；固定运营指标构成虚假产品事实，必须删除或封闭在明确 demo fixture。 |

## 6. P1 重判

### 6.1 Observation Session、Clock 与 Transport

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-1 | Open | 增加稳定 `ObservationSourceId`、process UUID/PID、target role、runtime generation 与 project/world identity。 |
| P1-2 | Open | Editor/Runtime 各自 `Instant` origin；增加 clock domain、base time、calibration sample、误差与不可比较状态。 |
| P1-3 | Open | 建立 connect/restart/disconnect/terminate lifecycle 和 generation-fenced source registry。 |
| P1-4 | Open | capture/query/export/reset 增加 request id、deadline、idempotency、terminal receipt，停止只返回状态字符串。 |
| P1-5 | Open | ABI 增加 source capability discovery，pane 不得默认宣称 watch/GPU/export/series 全可用。 |
| P1-6 | Open | snapshot 增加 collected_at、age、freshness、stale/disconnected/error/partial。 |
| P1-7 | Open | `ProfileControlResponse.status/message` 改 typed code/source/retryability/detail。 |
| P1-8 | Open | render/physics/animation/store/profile 增加同一 collection boundary 与 provider generation。 |
| P1-9 | Open | 完整 JSON response 改 bounded page/delta/full-resync，带 sequence/schema/depth/bytes budget。 |
| P1-10 | Open | child FFI unavailable/error 当前被 diagnostics helper静默回退；错误必须成为 pane 可见事实。 |

### 6.2 Metric Registry、Collector 与 Snapshot Store

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-11 | Open | 写入前注册 `MetricDescriptor`，未知或 metadata 冲突写入应拒绝并计数。 |
| P1-12 | Open | string path 拆成 dense `MetricId`、稳定 canonical path 与 display label。 |
| P1-13 | Open | unit/kind/aggregation/version 固定到 descriptor，不能在 record 时覆盖。 |
| P1-14 | Open | tags 当前永久 union；增加 label schema、数量/长度/bytes cap 与拒绝 receipt。 |
| P1-15 | Open | 增加 owner lease/deregister，plugin unload/world close/session end 清理 provider。 |
| P1-16 | Open | history=64 不能代替全局 series/cardinality/bytes/owner budget。 |
| P1-17 | Open | path/tag/unit 增加编码后 bytes 与字符串长度 admission。 |
| P1-18 | Open | `DiagnosticStore::record_measurement` 接受 NaN/Inf；finite policy应在写入边界执行。 |
| P1-19 | Open | 固定 0.9/0.1 EMA 无时间语义；改 measurement-time + descriptor time constant。 |
| P1-20 | Open | min/max 是 lifetime extrema，history eviction 不重算 window；区分 lifetime/window aggregation。 |
| P1-21 | Partial | retention 已有 written/overwritten/retained/oldest/newest；仍缺 source/sequence/time/generation。 |
| P1-22 | Partial | `current_snapshot()` 避免 history/tags clone；UI/full ABI 路径仍深复制完整 store。 |
| P1-23 | Partial | authoritative store 写回减少派生临时 store；diagnostics mutex 下 snapshot/clone 仍在热路径。 |
| P1-24 | **Closed** | `collect_runtime_diagnostics` 已更新 authority store 再 snapshot；“每次空 store 导致 derived history 重置”的具体旧 bug 当前不成立。 |
| P1-25 | Open | provider 缺 cadence/cost/admission/last-success/collection-latency health contract。 |

### 6.3 Runtime Diagnostics、Console 与 Timeline

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-26 | Open | Runtime Diagnostics payload 仍只有 summary/status/detail strings，没有 series/history/source/freshness。 |
| P1-27 | Open | child payload producer真实存在，但 Editor 只请求 profile snapshot；建立 typed consumer/cache。 |
| P1-28 | Open | 无 host/child/server 多 source 枚举、选择、对比和 role 显示。 |
| P1-29 | Open | world query/watch 未接 Diagnostics provider，不能关联 selected entity/asset 与 metric generation。 |
| P1-30 | Open | watch 缺 typed address/schema/owner generation；禁止字符串 query 直接进入长期 pane state。 |
| P1-31 | Open | watch subscription 缺 rate/history/trigger/budget/backpressure。 |
| P1-32 | Open | Events tab 无真实 event provider，固定 `Events 1.2K` 必须移除。 |
| P1-33 | Open | remote inspect/edit 缺 capability、principal、provenance、audit；默认只读且 fail-closed。 |
| P1-34 | Open | break/step/stack/callsite 需要 debugger provider，不能用 profile span 文本冒充。 |
| P1-35 | Open | Console Diagnostics 应查询 Editor11 journal，不应复制另一套 fixture/list/filter authority。 |
| P1-36 | Partial | converter 有 visible-row clipping；producer仍对 frame/span/hotspot 各取 12 行，不是连续 timeline。 |
| P1-37 | Open | 缺 zoom/pan/range/marker/selection/hover semantics。 |
| P1-38 | Open | stream/category/name 只是字符串，缺稳定 process/thread/task/GPU queue track identity。 |
| P1-39 | Open | counter 只进入 summary/has_samples，没有 counter track、scale 或 graph。 |
| P1-40 | Open | CPU/GPU timestamp 无 correlation/calibration/error，不得直接比较或排序。 |
| P1-41 | Open | frame index 只在 recorder 本地有效，缺 source/generation/global sequence。 |
| P1-42 | Open | parent id 不表达 async flow/task/thread relation；需要 typed analysis edges。 |
| P1-43 | Open | hotspot 仅按字符串聚合，缺 source/callsite identity 与 symbolization。 |
| P1-44 | Open | merge 不保留每 recorder effective config/budget，retention append 后难解释。 |
| P1-45 | Open | reverse/take(12) 按 append 顺序展示，不按统一 timestamp/range query。 |
| P1-46 | Open | dispatch 先同步控制 Editor recorder，再同步 child FFI，无 prepare/ack/timeout/cancel/rollback。 |
| P1-47 | Open | child 控制失败时 host recorder 可能已 stop/reset；需要双端 capture transaction。 |
| P1-48 | Open | export 对 caller root 多次直写固定文件，无 staging/manifest/hash/atomic publish/source isolation。 |
| P1-49 | Open | pane presentation 同步 snapshot、hotspot 分析和 FFI；改后台 job + generation cache。 |
| P1-50 | Open | 缺 `TraceStore` + typed analysis providers，pane builder 直接从 `ProfileSnapshot` 拼字符串。 |

### 6.4 Telemetry、Plugin 与资格

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-51 | Open | 无 telemetry event schema registry/provider；Dashboard 不得显示静态 DAU。 |
| P1-52 | Open | 无 consent/classification/redaction；diagnostic/profile 数据不可默认外发。 |
| P1-53 | Open | 无 endpoint/auth/tenant/environment identity。 |
| P1-54 | Open | 无 ingest/offline queue/delivery receipt；queued 文本不是投递。 |
| P1-55 | Open | 无 retention/deletion/aggregation policy；运营和本地诊断生命周期必须分离。 |
| P1-56 | Open | Crash Rate 无 crash evidence source；固定 0.18% 必须删除或显示 unavailable。 |
| P1-57 | Open | plugin 引用不存在的 `plugins://runtime_diagnostics/editor/authoring.zui`。 |
| P1-58 | Open | plugin 与 builtin 同用 `editor.runtime_diagnostics`；必须单一 view owner。 |
| P1-59 | Open | plugin test 只验证注册字符串，缺真实 package resource/load/owner receipt。 |
| P1-60 | Partial | retention/current snapshot/visible clipping/output budget/typed errors 提高局部质量；双进程、断线、规模、artifact、privacy资格仍空缺。 |

## 7. P2 重判

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P2-1 | Open | Diagnostics、Observability、Runtime Diagnostics 术语与 provider/subject 未统一。 |
| P2-2 | Open | internal 与 ABI `RuntimeDiagnosticsSnapshot` 分离，需 versioned public projection。 |
| P2-3 | Open | status/message/feedback 大量自由字符串，需 typed outcome + localization presentation。 |
| P2-4 | Open | `MAX_TIMELINE_ROWS=12` 是 producer 固定策略，不能靠 converter clipping 掩盖。 |
| P2-5 | Open | output path 直接显示为 String，需 artifact id/manifest/source identity。 |
| P2-6 | Open | rows 缺 source-qualified jump/freshness/subject target。 |
| P2-7 | Open | timeline source/filter/track/range preference 无持久化合同。 |
| P2-8 | Open | keyboard、screen reader、range/zoom/selection accessibility 未定义。 |
| P2-9 | Open | live/current/captured/stale/partial 视觉与交互语义未分离。 |
| P2-10 | Open | saturating span-id offset 必须删除，不能换更大整数继续伪合并。 |
| P2-11 | Open | plugin maturity/capability 没有传播到 view availability。 |
| P2-12 | Partial | owner/reference/currentness 边界已记录；production 模块合同仍未形成。 |

## 8. 参考引擎对照

| 参考 | 本地源码可验证机制 | Zircon 应吸收的边界 |
|---|---|---|
| Unreal TraceServices / TraceInsights | `IAnalysisSession` 有 trace id、session duration/base time、metadata、analyzer/provider registry、read/edit scope；`IAnalysisService` 区分完成分析与 streaming analysis；Insights 有 session changed/completed event，Timing View 有独立 session/track/time marker/relation。 | Observation Session、provider ownership、读写 scope、base time、analysis completion、stable track/marker/selection；不复制 Slate 实现。 |
| Godot debugger/profiler | `EditorDebuggerPlugin` 持有多个 `EditorDebuggerSession`，有 started/stopped/message/capture/profiler toggle；Debugger Node表达 remote tree、break/disconnect和session id；Performance profiler按 monitor 管理数据。 | 多 source lifecycle、断线/暂停、message capture、profiler capability与UI状态分离。 |
| Bevy diagnostics/remote | measurement 带 time，history/EMA/max length/NaN处理明确；Remote protocol有 request id、method、result/error、watch、discovery，HTTP transport独立插件。 | time-aware measurement、bounded registry、typed method/result/error/discovery；transport与UI解耦，不能照搬默认HTTP安全策略。 |
| Fyrox | Engine持有 PerformanceStatistics、elapsed time、scene graph；Graph暴露last-update performance stats，Editor stats window读取真实 renderer/scene/memory事实。 | domain owner、scene identity、真实窄统计面；不能把文本 stats 当完整 Timeline 上限。 |
| Unity Graphics | `DebugManager` 注册/注销 debug data、panel/widget和reset/dirty lifecycle；`IDebugDisplaySettings` 枚举/添加 settings；`DebugFrameTiming` 捕获CPU/GPU timing并注册有限 UI history。 | provider/panel lifecycle、reset/dirty、bounded timing history、runtime/editor capability；Graphics包不等于完整 Profiler 或 Analytics。 |

## 9. 分层重构计划

| Milestone | Owner / 产物 | 退出条件 |
|---|---|---|
| M0 Truthfulness | Editor workspace + plugin | 删除固定产品事实；无 provider 时显示 typed unavailable/demo；plugin resource可解析且 view owner唯一。 |
| M1 Observation Session | Editor + Interface | source/process/generation/project/world/role/capability/connection/freshness/last-error进入 registry snapshot。 |
| M2 Clock / Transport | Interface + Runtime | clock domain/base/calibration/error、request/deadline/idempotency、page/delta/resync 和 hard byte/item budget完成。 |
| M3 Metric Authority | Runtime03 | descriptor registry、dense id、finite/cardinality/bytes/label admission、owner lease和cadence health完成。 |
| M4 Editor Cache | Editor | child `runtime_diagnostics` 被后台 collector消费；pane只读 immutable generation cache，不做同步 FFI/full clone。 |
| M5 Trace Analysis | Runtime + Editor | source-qualified TraceStore；CPU thread/task、GPU queue、counter、frame、async relation由typed provider输出。 |
| M6 Capture / Artifact | Runtime + Editor | prepare/start/stop/seal/finalize、多 source receipt、partial/degraded、staging/manifest/hash/atomic publish完成。 |
| M7 Timeline Product | Editor | range query、zoom/pan/marker/selection/counter track/virtualization；取消 producer 12-row cap并提供规模证据。 |
| M8 Console / Plugin | Editor11 + Plugin | Console只查询canonical journal；plugin provider带capability/maturity/lease/load receipt。 |
| M9 Telemetry Governance | 独立Telemetry owner | schema、consent、classification/redaction、auth/tenant、retention/deletion、offline delivery和crash evidence全部通过后才开放。 |

## 10. 资格门

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01-G03 | Fail | fixture truthfulness、duplicate owner、missing plugin resource均未关闭。 |
| G04-G14 | Fail | source/clock/session/request/capability/snapshot/registry/cardinality/value/lease均无完整合同。 |
| G15 | Partial | per-series/ring/output budget和current snapshot存在；端到端总budget未证明。 |
| G16-G20 | Fail | presentation仍sync query/clone/FFI，freshness/child source/watch未闭环。 |
| G21 | Partial | profile retention有overwrite/oldest/newest，仍无统一sequence/drop/resync。 |
| G22 | Fail | Workbench Console未收敛到Editor11 journal。 |
| G23 | Partial | conversion能裁剪逻辑行；producer仍12行且无range/zoom/million-event evidence。 |
| G24-G31 | Fail | stable tracks/correlation/capture transaction/artifact/telemetry/privacy均未通过。 |
| G32 | Partial | 局部source-contract/retention/budget/error tests存在；跨进程/断线/规模/artifact/privacy未执行。 |

汇总为 **28 Fail / 4 Partial / 0 Pass**。

## 11. 禁止的临时修补

- 禁止只替换 fixture 数字、把 `MAX_TIMELINE_ROWS` 调大、继续偏移 span id，或吞掉 child FFI error 后显示 Live。
- 禁止在 pane presentation 增加更多同步 render/GPU query、完整 clone 或同步 FFI；先建立后台 cache 和 typed receipt。
- 禁止把 entity/request/asset/user input 拼成长期 metric path；禁止在没有 Telemetry governance 时连接 endpoint。
- 禁止复制日志、recorder、remote transport或builtin view来绕开canonical owner。
- 本轮只修改 review/index/coverage，不修改 production/test/Cargo/ABI；实施前必须重新冻结 baseline、fingerprint 和共享工作树状态。
