---
title: Editor Runtime Diagnostics / Performance Timeline / Console / Telemetry / Observability Authoring 当前源码复审
category: zircon_editor
report_id: Editor147
review_date: 2026-08-26
baseline_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
verification_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/99-editor-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/99zg-runtime-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/133-editor-logging-diagnostic-journal-output-console-status-routing-retention-export-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.zui
  - zircon_editor/src/core/gateway/session/profile.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders
  - zircon_editor/src/ui/retained_host/app/profiling
  - zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/runtime_diagnostics
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor147 - Runtime Diagnostics、Performance Timeline、Console 与 Telemetry 当前源码复审

## 1. 最终结论

Zircon 不是没有 diagnostics/profiling 基础。Runtime 已有有界 frame/span/counter recorder、retention evidence、热点分析、Perfetto/JSON 导出、render/physics/animation collector、per-series history、current-only snapshot、静态 metadata fast path、动态 ABI 输出预算与 child Runtime diagnostics DTO。Editor 也已有 builtin Runtime Diagnostics/Performance Timeline descriptor、可见性门控、pane payload、10,000 logical rows 的可见区裁剪、profile control 和 UI Debug Reflector。这些真实底层必须保留。

但它们没有形成工程级 observation product。四个 Workbench diagnostics surface 仍为 804 行、108 nodes、76 个 route assignment、0 provider，固定展示 `Session_Player_01`、`Frame 1234`、`Actors 420`、`DAU 128K`、`Crash Rate 0.18 percent` 与 `Events 2.4M`；callback 继续返回 queued/selected sample 文本。Console Diagnostics 也没有查询 Editor133 的真实 journal，仍是第二份静态事实源。

跨进程链路仍在关键入口断开。Runtime 的 `ProfileControlCommand::RuntimeDiagnosticsSnapshot` 能返回 project、scene、device、input、reload、diagnostic series 和 profile；但 Editor 的 `runtime_diagnostics_with_profile()` 只读取 host `EditorManager::runtime_diagnostics()`，对子 Runtime 发送普通 `Snapshot`，只取 `response.snapshot`。child diagnostics DTO 没有 Editor consumer，FFI error/None 被静默吞掉，UI 无法表达 source、disconnected、stale、partial 或 last error。

跨进程 profile 合并仍不可信。Editor 与 Runtime recorder 各自以本地 `Instant::now()` 为 origin，`ProfileSnapshot` 没有 source/process/runtime generation、clock domain、base time、calibration 或误差。`snapshot_merge.rs` 只做 saturating span-id offset、状态 OR、vector append、retention append 与 session string 拼接；不同进程事件因此被伪装成同一时间线。Performance producer 仍对 frame/span/hotspot 各自 `take(12)`，counter 只进入 summary，未形成可查询 track。

observer effect 也未关闭。可见 pane 在 presentation 收集路径同步查询 render/physics/animation、在 diagnostics mutex 下构造 full history snapshot、clone profile ring 并同步调用 child FFI。`current_snapshot()`、static metadata fast path、authority store 写回、可见性/刷新门控和 10K 行 UI clipping 是有价值的局部降本，但不能替代 fixed-cadence background collector、immutable generation cache、range query 和 stale/error projection。

`DiagnosticStore` 仍允许任意字符串路径隐式创建 `BTreeMap` series；只有单 series 64 点历史，没有总 series/path/tag/unit/bytes、owner lease、descriptor conflict、finite value 或 collection cost admission。Telemetry 生产代码搜索只命中静态 Dashboard/feedback，不存在 provider、event schema、consent、classification/redaction、endpoint/auth/tenant、offline delivery、retention/deletion 或 crash evidence。`runtime_diagnostics` plugin 仍与 builtin 共用 `editor.runtime_diagnostics`，且引用不存在的 `plugins://runtime_diagnostics/editor/authoring.zui`。

本轮重判 Editor25/99 的 **6 项 P0 为 5 Open/1 Partial，60 项 P1 为 54 Open/5 Partial/1 Closed，12 项 P2 为 11 Open/1 Partial；32 项资格门为 28 Fail/4 Partial**。Editor147 只刷新 currentness，不重复增加 canonical finding。没有运行真实 Editor、双进程采集、断线重连、长时 trace、GPU correlation、artifact fault、Telemetry privacy 或竞争 benchmark，不能声称表现或性能优于 Unreal。

## 2. 审查边界与 currentness

### 2.1 当前物理选择集

各范围是独立 owner 边界，存在有意重叠，不直接相加为 union。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮用途 |
|---|---:|---|
| 静态 Workbench 与 callback 边界 | **6 / 1,430 / 1,305 / 81,239 / 0 / 0** | 四份 ZUI、固定 feedback 与 navigation spec |
| Editor observability selected | **22 / 2,990 / 2,753 / 105,486 / 18 / 0** | builtin pane、gateway、visibility、profile merge/action 与转换 |
| Runtime diagnostics/profiling selected | **71 / 14,389 / 13,629 / 472,141 / 97 / 0** | store、collector、recorder、export、ABI DTO 与 output budget |
| `runtime_diagnostics` plugin | **9 / 436 / 395 / 16,128 / 4 / 0** | package、descriptor、resource URI 与 tests |
| Selected reference union | **17 / 11,021 / 9,830 / 406,228 / 16 / 0** | Unreal/Godot/Bevy/Fyrox/Unity Graphics 对照 |

### 2.2 冻结点与限制

- baseline HEAD 为 `166720dcb59c57fb4b33c34b859dc1a3f572b222`；审查以 dirty working tree 的物理内容为准，最终 HEAD 由 `verification_head` 标识。
- Unity Graphics revision `a7e4c051d256a781ab362c64316b125a1e104694`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，四个 nested repo 均 clean；Unreal 以所选文件内容为准。
- 共享 working tree 存在大量用户或其他 Session 在途修改。本轮不回退、不覆盖，也不把未集成改动当作已通过资格。
- 按用户要求未查询、轮询、等待或实时跟踪协调器。
- 本轮只做静态 review；未运行 Cargo、Editor、Runtime child、采集/导出、断线、规模、soak 或 benchmark。

### 2.3 Owner 边界

- Editor147 负责 Observation Session registry、source/freshness projection、Timeline query/interaction、Console journal projection、capture UX 和 Telemetry authoring truthfulness。
- Runtime03 负责 metric registry/store、collector cadence、profile recorder、TraceStore/analysis provider、capture/export primitive 与动态 Runtime DTO。
- Editor133 负责 canonical log/diagnostic journal；Console Diagnostics 只能查询它，不能维护第二份日志或固定计数。
- Editor07 与 Runtime gateway owner 负责 child lifecycle、generation、transport/backpressure；Editor147 只消费 generation-qualified observation source。
- Plugin owner 负责 provider lifecycle/capability/maturity/resource resolution；builtin view ID 只有一个 owner。

## 3. 当前源码事实与断点

| 子链 | 当前真实基础 | 仍然断开的工程合同 |
|---|---|---|
| Workbench surfaces | 4 份 ZUI 有稳定 control/route identity | 804 行/108 nodes/76 routes/0 provider，固定 session/frame/actor/telemetry 事实 |
| Host diagnostics | render/physics/animation availability 与 UI Debug Reflector | 不展示 store series/history/source/freshness，presentation 同步采集 |
| Child diagnostics ABI | `RuntimeDiagnosticsSnapshot` 含 project/scene/device/input/reload/series/profile | Editor 从未发送 `RuntimeDiagnosticsSnapshot` 命令，也无 DTO consumer |
| Profile control | Start/Stop/Export/Reset/Snapshot 均有真实 handler | 先改 host 再同步 child，无 request ID/deadline/idempotency/rollback/terminal receipt |
| Profile merge | retention 分 recorder 保留，span parent 同步 remap | 不同 clock/source 直接 offset+append，session ID 用字符串拼接 |
| Metric store | per-series 64 history、current/min/max/EMA、current-only snapshot | arbitrary path 隐式注册；无 descriptor、owner、cardinality、finite/bytes admission |
| Collector | render/physics/animation 写回 authority store，旧 history 丢失 bug 已关闭 | 采样时间/consistency generation/provider health/cost 仍缺失 |
| Timeline producer | frame/span/hotspot payload 与 capture controls | 三类各最多 12 行；counter 无 track；无 range/zoom/pan/marker/selection |
| Timeline conversion | 10K logical rows 只 materialize clip 相交行 | 上游只给 12 行，不能证明 million-event query/paint 资格 |
| Export | native JSON、Perfetto、hotspot/counter/UI reports | `create_dir_all + fs::write` 多文件直接发布，无 manifest/hash/atomic seal/source isolation |
| Console | Editor133 有真实 bounded journal/delta/virtualized base | Workbench Console Diagnostics 不查询 journal，仍显示固定行和计数 |
| Telemetry | 无生产基础可保留 | provider/schema/privacy/auth/tenant/delivery/retention/crash evidence 全缺失 |
| Plugin | descriptor/contribution mechanism 存在 | 与 builtin 重复 view ID，resource URI 不存在，测试只断言字符串 |

## 4. 必须保留的工程基础

1. 保留 Runtime `DiagnosticStore` 的 authority write-back、per-series history、current-only snapshot、static metadata fast path和 render/physics/animation availability；在其前面增加 descriptor/owner/admission，而不是复制 store。
2. 保留 child Runtime ABI 已有的 project、scene、device、input、reload、series 与 profile DTO；缺口是 version/source/freshness 和 Editor consumer，不是重新发明第三份 payload。
3. 保留 profiler frame/span/counter rings、retention sequence、hotspot、Perfetto/JSON、finite budget normalization；将每个 recorder 暴露为 source-qualified track，禁止继续物理伪合并。
4. 保留 pane visibility/refresh gate、10K row clip conversion、UI Debug Reflector 和 Editor133 journal；它们应成为 immutable cache/query 的终端。
5. 保留 plugin descriptor/extension 机制，但 builtin view 必须单 owner，plugin 只能注册真实 provider/extension，并带 owner generation、capability、maturity 与 revoke receipt。

## 5. P0 重判

| ID | 当前状态 | 当前证据与必须重构 |
|---|---|---|
| P0-1 | **Open** | 4 个 ZUI workspace 仍为 804 行、108 nodes、76 routes、0 provider；固定 session/metrics/feedback 必须改真实 provider 或显式 Demo/Unavailable。 |
| P0-2 | **Open** | `runtime_diagnostics_with_profile()` 只消费 host diagnostics 与 child profile snapshot；child Runtime diagnostics ABI 无 Editor consumer，必须建立 source-qualified projection。 |
| P0-3 | **Open** | `ProfileSnapshot` 仍无 source/process/clock/base/generation，merge 仍做 span offset、OR、append 与 session 拼接；必须改为多 source track。 |
| P0-4 | **Partial** | authority store、current-only periodic snapshot、visibility gate、ring capacity、finite budget 与 10K clip 减少局部成本；visible presentation 仍同步 query/full clone/profile FFI。 |
| P0-5 | **Open** | `BTreeMap<DiagnosticPath, DiagnosticSeries>` 仍允许 arbitrary path，只有 history 64；总 series、metadata/cardinality、owner lease 与 finite value 未受控。 |
| P0-6 | **Open** | 生产搜索无 Telemetry provider/schema/governance/delivery，只有固定 DAU/Crash Rate/Event 文案；静态运营事实必须删除。 |

## 6. P1 重判

### 6.1 Observation Session、Clock 与 Transport

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-1 | Open | `ProfileSnapshot` 无 stable source/process/runtime generation；增加 `ObservationSourceId`、PID/process UUID、target role 与 project/world identity。 |
| P1-2 | Open | Editor/Runtime 各自 `Instant::now()` origin，无 clock domain/base/calibration/error；未校准源不得排序或聚合。 |
| P1-3 | Open | 只有 visibility、dynamic session 与 recorder active，缺 connect/restart/disconnect/terminate observation lifecycle；建立 generation-fenced registry。 |
| P1-4 | Open | capture/query/export/reset 是同步命令与状态字符串，无 request/deadline/idempotency/terminal receipt；改 typed coordinator protocol。 |
| P1-5 | Open | Dynamic ABI 没有 source capability negotiation；UI 不应默认宣称 series/watch/GPU/export 可用。 |
| P1-6 | Open | Runtime snapshot 无 collected_at、source、freshness、age、stale/disconnected/error；pane 必须显式投影。 |
| P1-7 | Open | `ProfileControlResponse.status/message` 与 action status 仍为自由字符串；稳定 error code/source/retryability 必须进入公共接口。 |
| P1-8 | Open | render/physics/animation/store/profile 在不同时间点采集，snapshot 无 provider generation/consistency boundary。 |
| P1-9 | Open | 传输仍为完整 JSON request/response；增加 bounded page/delta/full recovery、sequence、schema/version、bytes/depth budget。 |
| P1-10 | Open | child FFI error/unavailable 被 helper 静默吞掉，UI 回退 host snapshot；错误必须成为 generation-qualified 可查询事实。 |

### 6.2 Metric Registry、Collector 与 Snapshot Store

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-11 | Open | `DiagnosticStore::record` 可隐式建 series；先注册 descriptor，未知/冲突写入拒绝并计数。 |
| P1-12 | Open | string path 同时承担 identity/display/storage key；引入 dense `MetricId` 与稳定 display path。 |
| P1-13 | Open | unit 是可覆盖 String；descriptor 固定 kind/unit/version，变更必须新 generation。 |
| P1-14 | Open | tags 永久 union；缺 bounded label value、cardinality 与 eviction。 |
| P1-15 | Open | 无 owner lease/deregister；plugin unload、World close、session end 可留下 provider state。 |
| P1-16 | Open | history 64 不是全局 cardinality budget；series/path/tag/bytes/owner 总额需 admission。 |
| P1-17 | Open | path/tag/unit 无长度和 encoded bytes budget；动态输入不能成为无界 key。 |
| P1-18 | Open | hotspot 分析过滤非 finite 只保护分析输入，`DiagnosticStore::record` 仍接收 NaN/Inf。 |
| P1-19 | Open | EMA 固定按样本 0.9/0.1，measurement 没有 monotonic time；改 time-constant/descriptor aggregation。 |
| P1-20 | Open | min/max 是 lifetime extrema，history eviction 不重算 window 统计。 |
| P1-21 | Partial | recorder retention 有 written/overwritten/retained/oldest/newest；measurement 仍无 source/sequence/time/generation。 |
| P1-22 | Partial | `current_snapshot()` 避免周期日志复制 history/tags，但 full `snapshot()` 仍深复制，UI 可见路径仍请求全量 store。 |
| P1-23 | Partial | collector 在 `update_diagnostic_store` 内维护 authority，减少临时 store clone；snapshot 仍在 diagnostics mutex 下深复制。 |
| P1-24 | **Closed** | `collect_runtime_diagnostics` 已写回 render/physics/animation authority store 后再 snapshot；旧“派生 history 每次从空 clone 开始”具体缺陷保持关闭。 |
| P1-25 | Open | provider 无统一 cadence/cost/admission/last-success/collection latency；availability query 不是 health contract。 |

### 6.3 Remote Runtime、Console 与 Timeline

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-26 | Open | builtin pane 只有 summary/render/physics/animation/detail strings，不展示 store series/history/source/freshness。 |
| P1-27 | Open | child `runtime_diagnostics_response()` 生成真实 payload，Editor 只请求 profile；建立 typed consumer。 |
| P1-28 | Open | 无多 Runtime source 选择/对比或 host/child/server role；Observation Session registry 必须可枚举。 |
| P1-29 | Open | world query/watch 未接 diagnostics provider，不能关联 selected asset/node/entity 与 metric generation。 |
| P1-30 | Open | watch 无 typed address/schema/owner generation；禁止把字符串 query 直接嵌入 pane。 |
| P1-31 | Open | 无 rate/history/trigger/budget 的 watch subscription；必须走后台 bounded stream。 |
| P1-32 | Open | Events tab 没有真实 event provider；固定事件数不可保留。 |
| P1-33 | Open | remote inspect/edit 权限、provenance、audit 未表达；读写能力必须 capability-gated。 |
| P1-34 | Open | break/step/stack/callsite 无 debugger provider；不得用 profile span 文本冒充 callstack。 |
| P1-35 | Open | Workbench Console Diagnostics 不查询 Editor133 journal；必须改 canonical journal projection。 |
| P1-36 | Partial | producer 仍对 frame/span/hotspot 各取 12 行；converter 的 10K clip 只保留为 virtualization foundation。 |
| P1-37 | Open | 无 zoom/pan/range/marker/selection 语义；固定列表不能推导时间轴交互。 |
| P1-38 | Open | stream/category/name 只是字符串，缺 stable track/thread/process/queue identity。 |
| P1-39 | Open | counters 只影响 summary/has_samples，未形成 counter track/row/graph。 |
| P1-40 | Open | CPU/GPU timestamp 无 correlation、queue、calibration/error；不得直接比较。 |
| P1-41 | Open | frame 只有 recorder-local index；需要 source/generation/global sequence。 |
| P1-42 | Open | parent ID 不含 async flow/task/thread relation；analysis provider 需要 typed edges。 |
| P1-43 | Open | hotspot 按 stream/category/name/path 字符串聚合，未带 source/callsite identity。 |
| P1-44 | Open | merge 无 per-source budget/config identity，retention 只 append，无法解释 effective budget。 |
| P1-45 | Open | producer `rev().take(12)` 按 append 顺序展示，不按统一 timestamp/range。 |
| P1-46 | Open | dispatch 先同步控制 Editor recorder，再同步 child FFI，无 prepare/ack/timeout/cancel/rollback。 |
| P1-47 | Open | reset/stop 可先改变 host 再 child 失败；现有局部 epoch 不进入双进程 capture transaction。 |
| P1-48 | Open | export 是 `create_dir_all + fs::write` 固定多文件，无 manifest/hash/atomic publish/source isolation。 |
| P1-49 | Open | pane presentation/command path 同步 snapshot、分析和 FFI；必须改后台 job + immutable cache。 |
| P1-50 | Open | 无 TraceStore + typed analysis provider，pane builder 直接从 `ProfileSnapshot` 拼字符串。 |

### 6.4 Telemetry、Plugin 与资格

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P1-51 | Open | 无 telemetry event schema registry/provider；Dashboard 不得显示静态 DAU。 |
| P1-52 | Open | 无 consent/classification/redaction；runtime/profile 本地数据不可默认外发。 |
| P1-53 | Open | 无 endpoint/auth/tenant/environment；UI 字符串不能成为服务。 |
| P1-54 | Open | 无 ingest/offline queue/delivery receipt；“Filter Telemetry”反馈不是投递。 |
| P1-55 | Open | 无 retention/deletion/aggregation policy；运营数据与诊断数据生命周期必须分离。 |
| P1-56 | Open | Crash Rate 没有 crash evidence source；固定 0.18% 必须删除或标 Unavailable。 |
| P1-57 | Open | plugin 引用 `plugins://runtime_diagnostics/editor/authoring.zui`，物理资源不存在。 |
| P1-58 | Open | plugin 与 builtin 共用 `editor.runtime_diagnostics`；必须单一 owner，plugin 只能 provider/extension。 |
| P1-59 | Open | plugin catalog/descriptor/resource resolution 语义矛盾，测试只断言字符串；需要 package load report。 |
| P1-60 | Partial | retention、current snapshot、visible-row clipping、finite/ring/output budget 与局部 tests 是真实基础；双进程、断线、规模、artifact、privacy 未闭环。 |

## 7. P2 重判

| ID | 状态 | 当前重构边界 |
|---|---|---|
| P2-1 | Open | Diagnostics、Debug Observatory、Runtime Diagnostics 缺统一 provider/subject/terminology。 |
| P2-2 | Open | internal Runtime diagnostics 与 ABI payload 分离，需公共 versioned projection。 |
| P2-3 | Open | status/message/feedback 大量 String 拼接；改 typed outcome + localization/presentation。 |
| P2-4 | Open | `MAX_TIMELINE_ROWS=12` 是 producer 固定策略，converter clipping 不能取消它。 |
| P2-5 | Open | artifact/output path 直接落 UI String，需 artifact id/manifest/source identity。 |
| P2-6 | Open | diagnostic rows 无 source-qualified jump/freshness/subject target。 |
| P2-7 | Open | timeline source/filter/track/range preference 没有持久化合同。 |
| P2-8 | Open | accessibility、keyboard range/zoom/selection 未定义。 |
| P2-9 | Open | live/current、captured、stale、partial 的视觉语义未分离。 |
| P2-10 | Open | `snapshot_merge` 仍用 saturating span-id offset；必须删除伪合并。 |
| P2-11 | Open | plugin maturity/capability 没有传播到 view availability。 |
| P2-12 | Partial | 本报告补充五套参考边界、owner 分工与 currentness 纪律；源码仍未满足这些边界。 |

## 8. 参考引擎对照

| 参考 | 当前源码可验证机制 | Zircon 应吸收 |
|---|---|---|
| Unreal TraceServices / TraceInsights | `IAnalysisSession` 有 Stop/Wait、duration、base date/time、metadata、BeginRead/BeginEdit 与 provider registry；Timing View 有 track、marker、selection、relation。 | observation session、provider ownership、read/edit scope、base time、analysis completion、stable track/marker/selection；不复制 Slate。 |
| Godot debugger | debugger node 管理多 session、active/breaked/disconnected 与 message channel；performance/profiler 分开控制 capture/history。 | 多 session lifecycle、断线/暂停、profiler capability 与 UI 状态分离；下拉字符串不能充当 authority。 |
| Bevy diagnostics/remote | DiagnosticPath/registration、measurement `Instant`、history/EMA/enabled/bounded length 明确；Remote 的 JSON-RPC schema/error/discovery 与 HTTP transport 分层。 | registry、time-aware smoothing、disabled fast path，以及 transport/method/error/UI 解耦；HTTP 默认策略不能直接照搬。 |
| Fyrox | engine 持有 PerformanceStatistics、elapsed time、task pool 与 scene graph，stats surface 是窄而真实的 runtime 下限。 | domain ownership、elapsed/frame stats、scene identity；文本 stats 不能冒充 Timeline 上限。 |
| Unity Graphics | `DebugManager` 可注册/注销 debug data，panel create/replace/remove、dirty refresh/reset 明确；`DebugFrameTiming` 保留有限 frame timing history。 | provider/panel lifecycle、reset/dirty、bounded timing history、runtime/editor capability；Graphics 包不能证明完整 Profiler/Analytics。 |

## 9. 分层重构路线

1. **M0 Truthfulness**：四个静态 Workbench surface 改真实 provider 或显式 Unavailable；删除固定 DAU/Crash Rate/Session/Frame/Actors 与 queued 假 receipt；修复 plugin resource/owner 冲突。
2. **M1 Observation Session**：建立 Editor-owned registry；source/process/runtime generation、project/world/target role、capability、connection/freshness/last error 进入统一 projection。
3. **M2 Clock/Transport**：ABI/profile/diagnostic DTO 增加 source、clock domain/base/calibration、sequence、schema、request/deadline/idempotency；page/delta/full recovery 有界。
4. **M3 Metric Cache**：MetricDescriptor registry、owner lease、cardinality/bytes/finite admission；Runtime fixed cadence 发布 immutable generation，Editor pane 只读 cache。
5. **M4 Runtime Diagnostics**：Editor 真实消费 child diagnostics DTO，展示 series/history/current/source、collected_at、age、partial/stale/error；host/child 不再互相冒充。
6. **M5 Trace Analysis**：source-qualified TraceStore/providers；CPU thread/task、GPU queue、counter、frame、async flow、journal relation 通过 stable track/edge 输出。
7. **M6 Capture/Artifact**：prepare/start/stop/seal/finalize transaction；per-source receipt、partial/degraded、epoch/generation、staging/manifest/hash/atomic publish 与 source-isolated path。
8. **M7 Timeline**：取消 producer 12-row cap，按 range/query 分页；实现 zoom/pan/marker/selection、counter track、virtualized rendering 与 million-event budget。
9. **M8 Console/Plugin convergence**：Console 只查询 Editor133 journal；Runtime/Timeline/Console 分别消费 canonical owner，plugin provider 带 capability/maturity/lease。
10. **M9 Telemetry**：只有 provider、schema、consent、redaction、auth、tenant、retention/deletion、offline receipt 与 crash evidence 全部通过后才开放。

## 10. 验收门禁重判

| Gate | 状态 | 说明 |
|---|---|---|
| G01/G02/G03 | Fail | fixtures、duplicate owner、missing plugin resource 均未关闭。 |
| G04-G14 | Fail | source/clock/session/request/capability/snapshot/registry/cardinality/value/lease 无完整合同。 |
| G15 | Partial | per-series history、ring/output budgets、current-only snapshot 存在；端到端采集预算未证明。 |
| G16-G20 | Fail | presentation 仍 sync query/clone/FFI；freshness、runtime source、watch 未闭环。 |
| G21 | Partial | profile retention 有 overwrite/oldest/newest，仍无统一 event sequence/drop stream。 |
| G22 | Fail | Workbench Console Diagnostics 未收敛到 Editor133 journal。 |
| G23 | Partial | conversion 可裁剪 10K logical rows，producer 仍为 12 rows；无 range/million-event evidence。 |
| G24-G31 | Fail | stable tracks/correlation/capture transaction/reset/artifact/telemetry/privacy 未通过。 |
| G32 | Partial | 局部 source-contract/retention/finite/ring tests 存在；断线、双进程、规模、artifact collision、missing resource、privacy 未执行。 |

## 11. 禁止的临时修补与本轮边界

- 禁止只替换 fixture 数字、提高 `MAX_TIMELINE_ROWS`、继续偏移 span id，或吞 child FFI 错误后显示 Live。
- 禁止在 presentation 增加更多 render/GPU query、full clone 或同步 FFI；先建立后台 collector/cache 与 typed receipt。
- 禁止把 entity/request/asset/user input 拼成长期 metric path；禁止在没有 Telemetry governance 时连接 endpoint。
- 禁止复制日志、recorder、remote transport 或 plugin ZUI 绕过 Editor133、Runtime03 与 gateway canonical owner。
- 本轮只完成 review/currentness 文档，未修改生产代码。实施前必须重取源码、重新冻结 source identity，并逐层执行双进程/fault/scale/soak/benchmark 资格。
