---
title: Runtime Process Diagnostic Log Router / Filter / Record / Queue / Sink / Durability / Rotation / Crash / Multi-Session 当前源码复审
category: zircon_runtime
report_id: Runtime132
review_date: 2026-08-24
baseline_head: 9199e18717e263c5b45cfeeb72d5c59fa061e68f
baseline_epoch: 391
verification_head: 9199e18717e263c5b45cfeeb72d5c59fa061e68f
verification_epoch: 391
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
related_code:
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/core/runtime/diagnostics/profiling
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/context
  - zircon_editor/src/ui
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/*/dist/src/lib.rs
tests:
  - zircon_runtime/src/diagnostic_log/diagnostics/tests
  - zircon_runtime/src/diagnostic_log/sink/tests
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/product_teardown.rs
  - zircon_editor/src/core/logging/tests.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-19-diagnostic-log-synchronous-sink.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zf-runtime-dynamic-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDeviceRedirector.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDeviceFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/OutputDeviceRedirectorTest.cpp
  - dev/bevy/crates/bevy_log/src/lib.rs
  - dev/godot/core/io/logger.h
  - dev/godot/core/io/logger.cpp
  - dev/godot/tests/core/io/test_logger.cpp
  - dev/Fyrox/fyrox-core/src/log.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/IDebugDisplaySettingsData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/Debugging/ProfilingSamplerTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/Debugging/ProfilingSamplerWithCommandBufferTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/DebugManagerTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime132 · Process Diagnostic Log Router / Filter / Record / Queue / Sink / Durability / Rotation / Crash / Multi-Session 当前源码复审

## 1. 结论

当前`zircon_runtime::diagnostic_log`不是占位实现。它有启动期编译的filter trie、lazy callsite、有界count FIFO、单worker批处理、flush/shutdown control、`sync_data`、queue age/drop/output error指标、panic hook链和动态session lease。当前源码还修正了Runtime44中两条已经过时的结论：Warn/Error满队列不再无限`send`，而是默认等待2ms后丢弃；flush/shutdown control也不再`try_send + yield`，而是按deadline调用`send_timeout`。普通测试已验证critical producer会在配置的10ms等待后返回，两个ignored性能入口分别覆盖54-case矩阵和2ms critical admission。

这些变化只把“必然无限阻塞”降成“有界但仍不合格”，没有建立工程级Process Log Control Plane。`LOG_CONTROLLER`仍是每个linked image自己的`OnceLock`；App静态链接的Runtime和动态Runtime DLL可各自建立一套router。`LogModule`仍只是descriptor，第一次初始化静默永久决定配置；API只返回`Option<PathBuf>`、`bool`或`()`，caller看不到Accepted、Dropped、Written、Synced或per-sink位置。record仍只有`level/scope/message/enqueued_at`，没有producer time、global sequence、thread/task/span/frame/session/project/plugin identity或结构化字段。队列只按条数有界，critical timeout可被公开settings设为任意大值，console和file仍共享单worker与故障域，control与data仍争用同一FIFO且没有sequence fence。

文件与崩溃路径仍远未闭合。process file没有rotation、retention、quota、exclusive create、manifest、reopen或platform crash artifact；worker以本地秒级目录和channel文件名append，整批共用worker生成的秒级时间戳，formatter只转义LF。panic hook在previous hook写panic payload之前先flush，所以process log本身没有panic record；abort、SEH、signal、OOM和hang没有协议。无输出配置虽然可建立active state，但`outputs_succeeded()`依赖存在sink，terminal API仍会返回false，说明初始化/有效状态/耐久结果尚未形成一致合同。

产品层继续分裂。Editor的独立`EditorLogService`已经有count+bytes store、sequence、有界event queue、resync和按day/size rolling file，这些是可复用的局部基础；但它仍在emitter线程、同一emission mutex内同步append/flush独立文件，不是统一router的journal view。Tracy只在feature开启时安装linked-image global subscriber，custom process log没有tracing layer。native plugin loader在entry期间确实提供`host_log`与`host_diagnostic` callback，但它们只写入全局临时capture再转成descriptor诊断；39个首方dist self-validation host table仍全部是`host_log: None`，运行期没有plugin identity、预算、sequence或router receipt。

Runtime44的52项P1本轮重判为 **39 Open、13 Partial、0 Closed**；14项P2为 **14 Open、0 Partial、0 Closed**；36项资格门为 **30 Fail、6 Partial、0 Pass**。Partial仅承认局部机制和静态测试存在，不代表产品、跨DLL、性能或耐久资格闭合。Runtime07的开放failure必须保持`open`，直到focused测试真实执行、54-case与critical benchmark产出有效artifact，并补齐rotation、shutdown、crash和真实filesystem证据。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径：按UTF-8物理行、非空行与文件bytes；Rust test declaration匹配`#[test]`/`#[tokio::test]`/`#[rstest]`，ignored按`#[ignore`计数；fingerprint为排序后的`relative-path|file-sha256`集合再做SHA-256。选择集既包含owner，也包含源码中实际出现`diagnostic_log`、`eprintln!`、`tracing::`、Editor log类型或native host log callback的产品消费者。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| DiagnosticLog完整owner与测试 | **31 / 4,121 / 3,688 / 134,682 / 45 / 2** | `46d26e78ef65c5734efa4f83bfea4c4a0e113210c797d4c7b0a8fa35437cda94` |
| Runtime集成，排除owner与plugin树 | **70 / 22,118 / 20,342 / 808,137 / 256 / 16** | `89bf240445e651dafce77b26fd5e21f82686164c056035e27334227605c07409` |
| App入口、CLI、teardown与消费者 | **30 / 8,765 / 8,093 / 340,126 / 111 / 0** | `fbf24db4e921d1f57de6361929e52ecf862be87d777a2b2242c841d75a2f49c7` |
| Editor日志owner与产品消费者 | **53 / 15,897 / 14,550 / 575,347 / 177 / 3** | `0105caf8f510fe390fa4ea06aaad531bc8f83f2ab465260b0af66054a4c5daa1` |
| Plugin ABI、SDK与发行描述 | **53 / 11,060 / 10,050 / 422,356 / 137 / 3** | `f13f6538108cd69833f196f9cb27617fde84c6d3824754e2af0fb601a649909d` |
| Zircon selected union | **237 / 61,961 / 56,723 / 2,280,648 / 726 / 24** | 由上面五个互斥组冻结 |
| Unreal/Bevy/Godot/Fyrox/Unity Graphics参考 | **18 / 5,522 / 4,781 / 194,910 / 0 / 0** | `045b87811d775ab0fdbec1f10a9bfbf3ac3da584d81edd34bab97fd6fb814509` |

### 2.2 检查方法

1. 全量逐文件读取`zircon_runtime/src/diagnostic_log`的生产代码、45个test declarations与2个ignored gates，而不是只做符号命中统计。
2. 沿App binary/CLI/EntryRunner、dynamic session acquire/drop、Runtime module catalog、profiling/Tracy、Editor context/UI、native plugin loader/SDK和39个dist描述追踪初始化、写入、flush、shutdown、panic及产品展示。
3. 对Runtime44的52项P1、14项P2和36个gate逐项重判；原编号保持不变，避免另造一套重复owner。
4. 对照Unreal redirector/output device/file/fence、Bevy tracing layer、Godot composite/rotation tests、Fyrox listener以及Unity Graphics profiling/debug-manager契约；Unity Graphics只用于其实际覆盖的render debugging/profiling，不外推Unity Player进程日志能力。
5. 运行repository-local Runtime结构审计；`audit_runtime_structure.py --json`在当前工作树以exit 0完成并生成33类结构结果。本轮不把全workspace结构审计当作日志资格通过证据。

### 2.3 currentness与共享工作树

- baseline为`9199e18717e263c5b45cfeeb72d5c59fa061e68f` / epoch 391；该baseline因共享工作树有3,206个未accept变更而标记degraded。
- 本轮读取到`diagnostic_log/level/compiled.rs`、`settings.rs`、`sink/tests/backpressure.rs`、`sink/tests/performance/critical.rs`、`sink/worker.rs`存在其他会话变更；最终重扫期间Runtime集成选择集仍被其他会话写入。本文记录表中最终重扫时点的物理指纹，不回退、不改写这些文件，也不承诺共享工作树随后停止变化。
- 本轮是review-only，不修改Runtime/App/Editor/Interface/Plugin源码，不运行Cargo，不提交commit。Cargo与真实产品执行不能由静态结论替代。
- `docs/plans/zircon_runtime/runtime/07/failure-2026-07-19-diagnostic-log-synchronous-sink.md`仍为canonical开放failure；本文只刷新事实和gate映射。

## 3. 必须保留的工程基础

1. 保留启动期编译的filter与lazy闭包入口；将byte-prefix扩展为segment-aware matcher、callsite metadata、动态generation与redaction compiler。
2. 保留bounded FIFO方向，但同时约束record count、encoded/owned bytes、per-owner额度和sink在途bytes。
3. 保留batch worker与queue age/high-water/drop指标，拆分router、encoder和各sink监督域。
4. 保留默认2ms critical enqueue timeout及普通bounded-return测试，但把timeout clamp、thread-domain预算、reserved emergency lane和typed admission纳入合同。
5. 保留显式flush/shutdown deadline与`sync_data`，升级为global cursor、per-sink fence和durability receipt。
6. 保留动态session lease，改成App-owned router lease；Runtime DLL不能继续拥有linked-image全局sink。
7. 保留panic hook链式委托previous hook的意图，先用panic-safe emergency writer记录payload，再做bounded fence。
8. 保留Editor的sequence、count+byte bounded store、有界event queue、resync和source/jump模型，把它改成统一record stream的journal projection。
9. 保留Editor rolling file已有day/size测试价值，但process artifact只能有一个file authority，迁移后删除Editor独立落盘。
10. 保留native plugin V3入口callback与panic guard，改成host-owned router submission并携plugin/package/module/entry generation及预算。
11. 保留Tracy可选layer与Runtime profile recorder，但日志、trace、metric必须共享event identity与clock anchor。
12. 保留真实write/flush/sync failure注入和54-case harness形状；补齐真实filesystem、process、crash、rotation、dual-image与平台矩阵。

## 4. 当前拓扑与断路

```text
zircon_app editor/runtime_preview
  -> static zircon_runtime LOG_CONTROLLER A -> one FIFO -> one worker -> stderr + file A
  -> load Runtime DLL
       -> dynamic session lease
       -> Runtime DLL LOG_CONTROLLER B       -> one FIFO -> one worker -> stderr + file B

zircon_editor EditorLogService -> bounded store -> synchronous event callback
                               -> synchronous RollingFileLogSink + flush -> editor file C

tracing::* -> optional linked-image Tracy subscriber

native plugin entry -> host_log/host_diagnostic -> temporary global capture
                    -> flattened descriptor diagnostic strings
plugin dist self validation -> 39/39 host_log: None
```

| 层 | 当前事实 | 尚未闭合的合同 |
|---|---|---|
| authority | `LOG_CONTROLLER: OnceLock<ProcessLogController>`位于Runtime crate映像 | 同一OS进程、App+DLL、多个session只有一个router generation |
| module | `LogModule`/`LogDiagnosticsModule`只提供descriptor且core候选固定包含Log | availability、provider实例、startup/shutdown receipt一致 |
| initialize | 第一次配置获胜，后续只写一条可能被filter隐藏的active/requested文本 | typed conflict/effective config、可重配policy、sink start ack |
| record | `level/scope/message/enqueued_at: Instant` | schema、producer timestamp、global sequence、context、fields、privacy |
| admission | best-effort先`is_full`再构造，Warn/Error最多等待配置timeout | byte cap、timeout clamp、thread budget、reserved lane、typed outcome |
| worker | 一个thread每batch取一次timestamp并串行写console/file | 原始发生时间、sink隔离、supervision、retry/reopen |
| control | `send_timeout`与data共用FIFO | monotonic fence、跨producer前序定义、per-sink durability cursor |
| file | 本地秒级目录、channel `.log`、create+append | exclusive identity、rotation、retention、quota、manifest、GC |
| crash | panic hook先flush再调用previous hook | panic payload、preallocated writer、SEH/signal/OOM/hang artifact |
| Editor | 本地service/store/event/rolling file独立且emit同步落盘 | 统一record订阅、异步projection、无重复落盘 |
| plugin | loader entry callback存在但只临时capture；39个dist table为None | 运行期callback/handle、identity、budget、sequence、receipt |
| diagnostics | 周期逐series格式化后逐条写log | typed atomic snapshot generation/completeness与sink health |

## 5. 相对Runtime44必须纠正的旧结论

| Runtime44旧结论 | 当前源码 | 本轮裁决 |
|---|---|---|
| Warn/Error满队列无限阻塞 | `send_timeout(command, critical_enqueue_timeout)`，默认2ms，普通测试用10ms并要求250ms内返回 | R44-P1-19、P1-51与G13升为Partial；公开timeout无上限、仍无thread-domain预算/receipt/reserve，所以不能Pass |
| control靠`try_send + yield`抢槽 | `send_control_until`直接按剩余deadline执行`send_timeout` | R44-P1-29与G20升为Partial；control仍和data共队列，缺global cursor，无法证明覆盖全部前序producer record |
| worker逐record生成timestamp | worker现在每个batch只生成一次timestamp | R44-P1-25升为Partial；所有batch记录时间相同且仍是worker time，R44-P1-10/G07继续Open/Fail |
| lazy callsite在队满时总会构造message | best-effort level在构造前检查`sender.is_full()` | R44-P1-12维持Partial；检查与`try_send`有竞态，closure仍可能构造后被丢弃，critical必先构造再等待 |
| plugin host log多数为None | production loader entry table传`Some(native_host_log_v3)`与`Some(native_host_diagnostic_v3)`；39个dist self-validation table仍全None | R44-P1-44/G33升为Partial；callback只在entry capture字符串，没有router/runtime identity/budget |
| Editor只有简陋第二套logger | 当前Editor service已有sequence、count+bytes store、event queue/resync、LogWriteReport和day/size rolling file | 作为projection底座保留；R44-P1-42/G31仍Open/Fail，因为它同步独立落盘且未消费统一stream |
| 测试把critical阻塞当作正确行为 | 普通backpressure test已要求bounded return/drop；54-case与critical benchmark为ignored | R44-P1-51升为Partial；Runtime07仍没有有效运行artifact，不能以测试源码形状关闭failure |

## 6. 五引擎参考证据与适用边界

### 6.1 Unreal Engine

`FOutputDeviceRedirector`明确区分primary/dedicated logging thread、backlog、同步/异步flush options和`CreateFence`；`FOutputDevice`声明sink能否跨线程、并发或panic thread使用；panic模式只保留声明panic-safe的output device。`FOutputDeviceFile`拥有async writer/circular buffer、周期flush、backup/collision retry和teardown，测试直接验证有/无logging thread时的fence。Zircon应借鉴“能力声明 + redirector + fence + panic-safe子集”，不能只复制一个后台线程。

### 6.2 Bevy

`bevy_log`以`tracing_subscriber::Registry`组合`EnvFilter`、fmt/custom layer、Tracy/Chrome/platform layer，并用`LogTracer`桥接`log`生态；global subscriber重复安装和filter parse failure有显式结果。Bevy源码也警告未连接profiler时Tracy可能无界buffer，说明Zircon必须对每个可选sink做内存资格，而不是把统一tracing等同于自动安全。

### 6.3 Godot

Godot把`Logger`、`StdLogger`、`RotatedFileLogger`和`CompositeLogger`分开；file logger包含backup数量、rotation、ANSI剥离和flush policy。`test_logger.cpp`使用真实`FileAccess`创建、轮转、删除旧文件并验证composite等价。Zircon process logger目前没有对应真实filesystem retention/rotation证据；Editor本地rolling不能替代process authority gate。

### 6.4 Fyrox

Fyrox的全局`Log`有listener sender、relative time、one-shot去重、stdout/file写入和flush。它可作为最小可用UX与subscriber语义参考，但同步mutex/file路径不是Zircon追求更高吞吐和故障隔离的目标上限。

### 6.5 Unity Graphics

`ProfilingSampler`只在`enableRecording`打开后建立CPU/GPU Recorder；`ProfilingScope`以RAII包围CommandBuffer marker；测试读取真实Recorder sample并按平台/GPU能力设置条件。`DebugManager`集中注册debug data/panel/widget、runtime/editor UI显示与reset。这些证据支持Zircon把structured diagnostic/profiling record投影到产品调试UI，并为marker做真实设备测试。该package不拥有Unity Player/Editor的process log、rotation或crash reporter，因此本文不从包内“未找到”推断Unity引擎缺失这些能力。

## 7. P0与唯一owner边界

本文不新增P0。以下父报告继续拥有canonical合同，Runtime132只记录其在process logging上的阻断：

| Owner | 唯一职责 |
|---|---|
| Runtime03 | canonical diagnostics schema/store/query、profile/config与metric sample |
| Runtime07 + open failure | producer hot path、bounded admission、performance、shutdown/crash/rotation资格 |
| Runtime43/Runtime131 | dynamic session lifecycle与host-provided service lease |
| App01 | ProcessHost、路径policy、early bootstrap、panic/crash coordinator和terminal exit |
| Interface01 | App/Runtime DLL版本化host service ABI、foreign lifetime与receipt |
| Editor11 | journal/query/output console/retention/export product projection |
| Plugins01 | plugin SDK/ABI capability、identity、callback lifetime与发行验证 |

## 8. Runtime44 P1当前状态

### 8.1 Authority、生命周期与多实例

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-01 | Open | `OnceLock`仍按linked image存在，App与Runtime DLL没有共享实例证明 | App创建`ProcessLogRouter`，V2 host service只传borrowed/leased submission面 |
| R44-P1-02 | Open | Log模块只有descriptor，未拥有provider生命周期 | module capability绑定实际provider generation与startup/shutdown receipt |
| R44-P1-03 | Open | 首次init静默获胜，后续差异只尝试写filtered文本 | `InitializeReceipt { disposition, generation, effective_config, diagnostics }` |
| R44-P1-04 | Open | init只返路径，write返`()`，flush/shutdown返bool；无output配置terminal还会false | 全API返回typed state/admission/fence/durability disposition |
| R44-P1-05 | Open | App静态init和DLL dynamic lease仍各自推导policy | App唯一冻结product/project/channel/path/filter与privacy policy |
| R44-P1-06 | Partial | dynamic lease有session计数并避免首个session提前shutdown；无session/host/project身份 | generation lease registry、owner census、stale/foreign/double-release拒绝 |
| R44-P1-07 | Partial | flush/shutdown/unload有deadline；Drop仍忽略失败或由App补偿abort，缺多映像terminal receipt | 统一`ShutdownBudget`与phase/per-sink terminal receipt，库不得杀宿主 |
| R44-P1-08 | Open | bootstrap/filter parse/spawn及大量产品路径仍直接`eprintln!` | 有界EarlySpool/EmergencyWriter，发布router后按sequence并入artifact |

### 8.2 Record、时间、filter与安全

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-09 | Open | record仍只有level/scope/message和queue Instant | versioned typed fields/callsite/schema ID，text/JSON/binary同源 |
| R44-P1-10 | Open | 每batch只取一个worker wall time，批内记录同秒且不是发生时间 | producer wall+monotonic time、clock domain/anchor与global sequence |
| R44-P1-11 | Open | 无process/thread/task/span/frame/session/project/plugin identity | 低成本context token与显式Unknown，不用字符串猜owner |
| R44-P1-12 | Partial | filter/lazy和best-effort `is_full` precheck减少部分构造；竞态下仍可构造后drop | static callsite、interned target、bounded arena及accepted/disabled分配门 |
| R44-P1-13 | Open | queue只按record count；scope/message String无单条和总bytes cap | per-field/record/owner/queue/process硬byte cap与truncate/reject receipt |
| R44-P1-14 | Open | formatter只替换LF；CR、ANSI、control和scope `]`可注入 | sink-specific escaping、ANSI policy、UTF-8 repair与字段边界语料 |
| R44-P1-15 | Open | compiled trie保持原始byte prefix，`asset`仍匹配`assets` | segment-aware target grammar、wildcard与canonical normalization |
| R44-P1-16 | Partial | parser拒绝空scope/未知level，sink数值至少归一到1；公开config仍可放重复/无界rule和任意timeout | 单一validated config compiler、rule/bytes/timeout上限与deterministic hash |
| R44-P1-17 | Open | `RUST_LOG`只解析自定义极小子集，一条非法directive会回退整份override | 完整实现承诺语法，或更名并逐directive返回诊断 |
| R44-P1-18 | Open | 无runtime reload、compile-time max、field privacy/redaction | 原子filter generation、static max level和release隐私policy |

### 8.3 Admission、queue、worker与监督

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-19 | Partial | Warn/Error改为默认2ms `send_timeout`后drop；timeout公开且无上限，critical仍共享FIFO | thread-domain budget、timeout clamp、reserved lane/emergency sink与receipt |
| R44-P1-20 | Open | enqueue bool仍只表示进入RAM，不表示Written/Flushed/Synced | `LogAdmission`与per-sink durability fence严格命名 |
| R44-P1-21 | Open | 单FIFO无owner/session配额、公平或critical reserve | per-owner token/bytes、公平scheduler和全局emergency reserve |
| R44-P1-22 | Open | public write丢弃`SinkRuntime::enqueue` bool | 返回Accepted/Dropped/Truncated/Degraded及聚合原因 |
| R44-P1-23 | Partial | 已有global depth/age/drop/write/bytes/batch/backpressure/error/closed指标 | 增per-owner/per-sink bytes、latency histogram、retry、generation和last error |
| R44-P1-24 | Open | `max_batch_bytes`用estimated bytes，首条oversize仍进入batch | actual encoded hard cap、oversize lane及checked accounting |
| R44-P1-25 | Partial | timestamp从逐record降到每batch一次；仍新建batch buffer且重写发生时间 | producer timestamp、复用bounded encoder arena并冻结alloc/CPU曲线 |
| R44-P1-26 | Open | console和file由同worker顺序调用 | 每sink独立queue/worker/budget/health，encoder共享但阻塞域隔离 |
| R44-P1-27 | Open | output error为sticky计数，无retry/reopen/failover/disable state machine | supervised sink lifecycle、退避与operator notification |
| R44-P1-28 | Open | worker panic没有catch/supervisor；close等待依赖metrics且busy-yield | panic boundary、terminal mark、join/restart policy与不可恢复receipt |
| R44-P1-29 | Partial | control发送改成deadline `send_timeout`；仍与data争用同FIFO且无global sequence | 独立control lane或monotonic fence cursor，只等待已定义前序 |

### 8.4 File、rotation、durability与crash

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-30 | Open | process file不rotation；Editor day/size rolling属于另一authority | process `RotatingFileSink`按size/time/session原子close/publish |
| R44-P1-31 | Open | 无retention、总quota、compression或GC | product/project/channel总预算、age policy和可中断GC |
| R44-P1-32 | Open | 本地秒级目录+append允许同秒/并发进程碰撞 | UTC高精度+PID+process generation+nonce且exclusive create |
| R44-P1-33 | Open | sanitizer仅保留ASCII alnum/`-`/`_`，有碰撞、长度和Windows保留名问题 | safe stem + stable hash、长度与platform reserved-name校验 |
| R44-P1-34 | Open | Runtime内硬编码company/product并猜exe/cwd/Unity user路径 | App冻结canonical root、portable/symlink/ACL policy并返回path receipt |
| R44-P1-35 | Open | artifact没有schema/build/commit/platform/process/project/config/clock manifest | session manifest及每segment identity/checksum |
| R44-P1-36 | Open | candidate/spawn失败走filter或`eprintln!` | 不可过滤bootstrap channel与emergency artifact |
| R44-P1-37 | Open | open/spawn失败可能遗留目录/空文件，requested file与effective state分裂 | transactional prepare/start/ack/publish，失败清理临时artifact |
| R44-P1-38 | Partial | explicit flush/shutdown会`flush + sync_data`并有错误测试 | 明确Buffered/OSFlushed/DataSynced/MetadataSynced等级及成本 |
| R44-P1-39 | Open | panic hook先flush，panic payload由previous hook之后写stderr | preallocated panic record先写emergency handle，再bounded fence并链式委托 |
| R44-P1-40 | Open | 无abort/SEH/signal/OOM/hang平台合同 | CrashArtifactCoordinator、external collector与平台资格矩阵 |

### 8.5 产品集成、diagnostics与一致性

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-41 | Open | App静态Runtime和动态DLL仍可各有router | Interface V2 host service提交同一record sequence与lease |
| R44-P1-42 | Open | Editor service虽更完整，仍独立同步store/event/file | Editor只保留journal/query projection，删除独立file authority |
| R44-P1-43 | Open | custom process logger不是tracing layer；Tracy单独安装global subscriber | 一个Registry桥接process log/profiler/platform layer与统一identity |
| R44-P1-44 | Partial | loader entry提供V3 callbacks并有panic guard；仅写临时global capture，39个dist self-table为None | 运行期host handle、plugin identity/capability/budget/receipt与router提交 |
| R44-P1-45 | Open | diagnostic store逐series转文本逐条入队，无snapshot generation/completeness | typed atomic batch或artifact reference，保留partial/drop disposition |
| R44-P1-46 | Open | schedule大delta继续`while elapsed >= period`重复减法 | O(1) miss count、coalesced/skipped periods和sample timestamp |
| R44-P1-47 | Open | metric被格式化为字符串，缺frame/tag validity，NaN/Inf无policy | typed metric schema/sample与non-finite policy，文本仅sink projection |
| R44-P1-48 | Open | sink health未进入canonical diagnostics、Editor或ABI | 发布router/sink generation、backlog/age/drop/error/durability/rotation |

### 8.6 测试与资格证据

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| R44-P1-49 | Partial | 54-case矩阵和shape test存在，但资格入口ignored且Runtime07尚无有效执行artifact | 固定managed validator命令、真实case count、平台与artifact hash |
| R44-P1-50 | Partial | harness采集caller p95、queue/RSS/worker指标；主要是instrumented output和粗RSS门 | p50/p95/p99/max、alloc/CPU、真实NVMe/慢盘/console与profile矩阵 |
| R44-P1-51 | Partial | 普通test已验证critical bounded return/drop，另有ignored 2ms benchmark | frame/job/UI producer硬预算、timeout clamp和typed degrade receipt |
| R44-P1-52 | Open | 无process rotation、真实FS、worker panic、dual-image、crash/platform测试 | unit/property/fault/process/integration/crash五层可归档套件 |

P1连续编号检查：01-52共52项；Partial为06、07、12、16、19、23、25、29、38、44、49、50、51，共13项；其余39项Open，无Closed。

## 9. Runtime44 P2当前状态

| ID | 状态 | 当前证据与必须改进 |
|---|---|---|
| R44-P2-01 | Open | 无compile-time event code/field schema；生成并做collision/upgrade测试 |
| R44-P2-02 | Open | 无binary chunk sink/offline index；与text/JSON共享record ID roundtrip |
| R44-P2-03 | Open | 无按owner/session动态sampling与重复折叠；保留first/last/count/critical |
| R44-P2-04 | Open | 无受privacy/consent控制的remote telemetry与有界spool |
| R44-P2-05 | Open | 无固定bytes、panic-safe读取的内存ring buffer |
| R44-P2-06 | Open | 无artifact seek table/time-sequence index |
| R44-P2-07 | Open | Editor/child runtime无认证、重连、背压IPC汇聚 |
| R44-P2-08 | Open | 无field-level release redaction token和secret corpus |
| R44-P2-09 | Open | 无CPU/GPU/remote clock calibration与reset/drift事件 |
| R44-P2-10 | Open | sink不能按capability热挂载/卸载并以fence收尾 |
| R44-P2-11 | Open | 无遵守size/privacy预算的support bundle日志/manifest/metric/crash导出 |
| R44-P2-12 | Open | Editor filter仍从内存snapshot扫描，无server-side query/index contract |
| R44-P2-13 | Open | 无有界字符串/field interning或compression及退化曲线 |
| R44-P2-14 | Open | 无24h/7d soak、artifact规模和磁盘写放大预算 |

P2连续编号检查：01-14共14项，全部Open。

## 10. 资格门当前状态

| Gate | 状态 | 当前证据与未闭环 |
|---|---|---|
| R44-G01 | Fail | App加载Runtime DLL后单router generation未证明 |
| R44-G02 | Fail | record没有session/project/world identity |
| R44-G03 | Fail | init冲突无typed disposition/effective config |
| R44-G04 | Fail | Log descriptor availability不等于实际provider |
| R44-G05 | Fail | early/terminal错误仍有`eprintln!`旁路且不保证入artifact |
| R44-G06 | Fail | 无versioned structured schema或text/JSON/binary同源 |
| R44-G07 | Fail | 无producer time、monotonic sequence或thread context |
| R44-G08 | Partial | filter/lazy/precheck是基础；disabled/accepted allocation预算未运行 |
| R44-G09 | Fail | record/field/queue总bytes不硬有界 |
| R44-G10 | Fail | CR/ANSI/control/UTF-8/scope注入语料缺失 |
| R44-G11 | Fail | byte-prefix边界和声明兼容矩阵不合格 |
| R44-G12 | Fail | 无atomic reload与release redaction |
| R44-G13 | Partial | critical默认2ms且有bounded test；timeout可任意大、无thread SLO/reserve |
| R44-G14 | Fail | public caller无admission disposition |
| R44-G15 | Fail | 无owner公平、quota和critical reserve |
| R44-G16 | Fail | batch actual encoded bytes无硬门 |
| R44-G17 | Fail | 慢console与file共享worker |
| R44-G18 | Fail | 慢file仍拖console并最终施压producer |
| R44-G19 | Fail | sink panic无supervisor/terminal proof |
| R44-G20 | Partial | control send有deadline；同队列且无global fence cursor |
| R44-G21 | Fail | 无per-sink Written/Synced receipt位置 |
| R44-G22 | Fail | process router没有size/time/session rotation |
| R44-G23 | Fail | 无retention/quota/GC长期有界证明 |
| R44-G24 | Fail | 同秒/并发进程/channel碰撞仍可共享文件 |
| R44-G25 | Fail | Windows保留名、长路径、symlink、ACL矩阵缺失 |
| R44-G26 | Fail | artifact无build/process/project/config/clock manifest |
| R44-G27 | Fail | init失败仍可被filter隐藏并遗留幽灵artifact |
| R44-G28 | Partial | explicit flush/shutdown执行`sync_data`；无metadata sync等级/fault model |
| R44-G29 | Fail | panic artifact不含payload，hook链只有静态形状 |
| R44-G30 | Fail | Windows/Linux/macOS crash/abort/OOM无平台证据 |
| R44-G31 | Fail | Editor仍独立落盘而非统一record journal |
| R44-G32 | Fail | tracing与custom log没有共享event identity |
| R44-G33 | Partial | native entry callback存在；运行期identity/budget/capability truth缺失 |
| R44-G34 | Fail | diagnostic snapshot无generation/completeness |
| R44-G35 | Partial | 54-case和critical ignored harness存在；没有有效非零运行artifact |
| R44-G36 | Fail | Runtime07 rotation/shutdown/crash/perf条件未全部满足 |

Gate连续编号检查：01-36共36项；Partial为08、13、20、28、33、35，共6项；其余30项Fail，无Pass。

## 11. 目标架构与owner边界

```text
zircon_app::ProcessHost
  owns ProcessLogPolicy + ProcessLogRouter + CrashArtifactCoordinator
       |
       +-- EarlySpool / EmergencyWriter (preallocated, panic-safe subset)
       +-- RecordCompiler
       |     schema + callsite + context + clocks + filter + redaction
       +-- AdmissionScheduler
       |     count/bytes/owner budgets + critical reserve + typed outcome
       +-- GlobalSequence / FenceCoordinator
       +-- SinkSupervisor
       |     +-- ConsoleSink worker/queue/health
       |     +-- RotatingFileSink worker/queue/manifest/retention
       |     +-- RingBufferSink
       |     +-- optional TelemetrySink
       +-- Canonical health publisher
       |
       +-- zircon_runtime_interface::ProcessLogHostServiceV2
              +-- Runtime DLL/session leases
              +-- plugin/package/module leases
              +-- tracing layer
              +-- Editor journal/query projection
```

| Owner | 唯一拥有 | 禁止继续拥有 |
|---|---|---|
| `zircon_app` | process policy、router实例、路径、crash coordinator、terminal shutdown | 自己格式化第二份Runtime日志 |
| `zircon_runtime::diagnostic_log` | schema/filter/admission/router/sink/fence实现 | 猜产品路径、决定process lifetime、linked-image第二全局实例 |
| Runtime DLL | session context与host-service lease | 初始化/关闭process sink |
| `zircon_editor` | journal/query/UI retention/export interaction | 独立process rolling file与同步I/O authority |
| plugin ABI | 有预算的structured record submission | 无身份stdout、临时global capture作为运行期日志 |
| Runtime03 | typed metric/profile/query | 文本artifact生命周期 |
| platform crash层 | panic/SEH/signal/OOM/hang捕获与emergency handle | 常规filter或产品UI策略 |

## 12. Hard Cutover约束

1. 不保留旧logger与新router双写兼容期；adapter先落地，生产caller一次性迁移后删除旧global initializer。
2. 不把默认2ms当作普适frame-safe预算；所有公开timeout必须clamp并按thread domain资格化。
3. 不把enqueue bool命名为durable；Accepted、Written、Flushed、DataSynced和MetadataSynced必须分离。
4. Runtime DLL、Editor和plugin不得创建第二个process file sink。
5. 除审计过的EarlySpool/EmergencyWriter外，常规生产路径不得直接`eprintln!`。
6. 不用compat shim保留byte-prefix filter或伪`RUST_LOG`承诺；配置迁移逐directive报告。
7. control不能继续靠“最终抢到同一FIFO槽位”冒充fence；必须以sequence cursor定义前序。
8. crash path不得无界分配、获取未声明panic-safe的锁或调用普通sink。
9. Editor rolling file迁移后删除，不能因其已有rotation而保留双artifact事实。
10. Runtime07 failure在所有focused/perf/rotation/shutdown/crash证据有效前不得关闭。

## 13. TDD实施里程碑

### M0：Truth freeze与失败语义

- 先写App+Runtime DLL同进程双router、init冲突、无输出terminal false、queue byte oversize、batch timestamp collapse和plugin entry-only capture的RED测试。
- 冻结`DiagnosticRecordV1`、`LogAdmission`、`InitializeReceipt`、`FenceReceipt`、`SinkHealth`与durability枚举。
- 把Runtime07剩余验收映射到G08/G13/G16/G20/G22/G28/G29/G35/G36，不改变failure状态。

### M1：App-owned单一authority与host ABI

- App创建router和policy，通过`ProcessLogHostServiceV2`把leased submission面传给Runtime DLL/plugin。
- RED测试覆盖同进程多DLL/session单sequence、stale lease、DLL unload与host先后顺序。
- 删除Runtime DLL内部init/final shutdown authority。

### M2：结构化record、filter与privacy compiler

- segment matcher、directive逐项错误、rule/bytes预算、callsite collision、control-char corpus先RED。
- producer捕获wall/monotonic time、thread/task/span/frame/session/project/plugin context。
- custom macros与tracing layer提交同一record identity。

### M3：字节有界admission与公平调度

- oversize record、noisy owner、critical reserve、frame/job/UI producer latency先RED。
- count+bytes+owner quota、timeout clamp、reserved lane和typed degrade receipt。
- best-effort lazy closure是否执行由真实reservation决定，删除racy `is_full`推断。

### M4：sequence fence、sink隔离与监督

- 慢stderr、慢file、write error、sink panic、reopen、满载flush与concurrent producer order先RED。
- 每sink独立worker/queue/health，router维护accepted/written/synced cursors。
- flush receipt逐sink报告达到位置、失败、超时和durability。

### M5：RotatingFileSink、manifest与retention

- 真实目录上先覆盖exclusive create、同秒多进程、size/time/session rotation、quota、permission、disk full和residue。
- segment manifest包含build/commit/platform/process/project/config/clock/sink generation与checksum。
- transactional prepare/start/ack/publish，后台GC可取消且有预算。

### M6：crash与emergency artifact

- panic payload/backtrace disposition、hook chain、timeout、worker-thread panic、abort/SEH/signal/OOM/hang按平台先RED。
- 预分配emergency buffer/handle，普通sink只参与声明panic-safe的bounded fence。
- external collector与support bundle只消费已发布artifact/manifest。

### M7：Editor、diagnostics、profiling与plugin收敛

- Editor journal单record不重复落盘、event gap resync、query/export和UI状态先RED。
- diagnostic snapshot用typed batch generation/completeness，sink health进入canonical store。
- plugin运行期callback携identity/capability/budget/receipt；39个dist self-validation也使用同一host test fixture。

### M8：资格与旧路径删除

- managed Windows执行focused tests、54-case矩阵、critical benchmark、真实filesystem/process/crash矩阵，保存case count与artifact hash。
- Linux/macOS只在对应平台需求下补齐crash/path资格。
- 删除旧global initializer、Editor rolling file、entry-only capture作为日志路径、常规`eprintln!`旁路和伪兼容filter。

## 14. 测试与证据矩阵

| 层 | 已有可保留证据 | 缺失的接受证据 |
|---|---|---|
| filter/lazy | parse、precedence、compiled equivalence、filtered/no-output/shutdown closure guards | segment boundary、rule budget、reload、redaction、allocation counters |
| queue/order | FIFO、batch、drop、bounded critical return | byte/owner bounds、fairness、reserve、multi-producer global sequence |
| output/durability | write/flush/sync error injection、explicit `sync_data` | real FS、disk full、metadata sync、per-sink cursor、crash model |
| lifecycle | concurrent shutdown、dynamic lease与product teardown source guards | App+DLL single authority、stale lease、DLL unload、no-output typed terminal |
| performance | 54-case shape、ignored matrix、ignored critical benchmark | managed nonzero run、p99/max/alloc/CPU、real disk/console、artifact hash |
| Editor | store bytes/count、event backpressure/resync、rolling day/size | unified stream、async projection、不重复落盘、query/index/retention |
| plugin | V3 callback ABI、panic guard、entry diagnostic conversion | runtime logging、identity/budget/receipt、dist host fixture、unload race |
| crash/platform | panic hook source shape | payload artifact、abort/SEH/signal/OOM/hang及三平台证据 |

本轮没有运行Cargo。review-only可以做静态状态裁决，但任何Gate从Partial/Fail升级到Pass都必须引用实际命令、目标test count、平台、revision和artifact hash。

## 15. 逐文件检查台账

### 15.1 DiagnosticLog完整owner与测试

| 文件/族 | 当前职责 | 裁决 |
|---|---|---|
| `diagnostics.rs`及`diagnostics/tests/{format_schedule,lazy_callsite_guards,ownership,mod}.rs` | diagnostic store周期格式化、schedule与lazy owner测试 | owner已拆分；逐series文本、非原子snapshot和O(delta/period)仍需重构 |
| `level.rs`、`level/compiled.rs` | env parser、公开config、byte-prefix trie | 编译基础保留；segment、rule budget、reload、privacy缺失 |
| `settings.rs` | queue/batch/flush/shutdown/critical timeout及诊断文本 | 默认2ms是真实修正；公开timeout/rule/byte policy没有全局validation |
| `platform.rs`、`timestamp.rs` | 目录候选、文件名和秒级timestamp | 路径policy越权、碰撞/manifest/高精度clock缺失 |
| `sink.rs` | linked-image controller、init、public write/flush/shutdown、panic hook | process authority和receipt的主要重构点 |
| `sink/metrics.rs` | global atomic sink counters | 基础保留；per-owner/per-sink/generation/histogram缺失 |
| `sink/worker.rs` | count FIFO、lazy enqueue、critical timeout、batch、fanout、control、sync | bounded修正保留；byte/fairness/fence/isolation/supervision未闭合 |
| `sink/tests/backpressure.rs` | normal bounded critical与ignored critical perf | 旧无限阻塞断言已纠正；thread SLO和typed outcome仍缺 |
| `sink/tests/{batching,durability,lifecycle}.rs` | batch、错误、sync、shutdown | 保留并扩成真实FS/process/fault矩阵 |
| `sink/tests/fixtures.rs`、`sink/tests/mod.rs` | BlockingOutput、shared settings与test wiring | 可作为fault fixture底座 |
| `sink/tests/performance/{case,configuration,critical,mod,output,pacing,report,resources,rss,rss/windows,validation}.rs` | 54-case、critical companion、pacing、RSS、报告 | 物理完整但2个gate ignored，未形成接受artifact |
| `diagnostic_log/mod.rs` | public exports与module docs | docs仍不能替代typed contract和产品资格 |

31个文件物理行分别为：`diagnostics.rs`154、format_schedule 62、lazy_callsite_guards 71、tests/mod 3、ownership 73、`level.rs`400、`level/compiled.rs`68、`mod.rs`38、`platform.rs`138、`settings.rs`239、`sink.rs`687、`metrics.rs`138、backpressure 225、batching 71、durability 54、fixtures 133、lifecycle 243、sink tests/mod 58、performance case 108、configuration 56、critical 73、performance/mod 48、output 101、pacing 26、report 166、resources 46、rss 93、rss/windows 41、validation 69、worker 436、timestamp 3。总计4,121行。

### 15.2 Runtime与App集成

| 文件/族 | 当前事实 | 裁决 |
|---|---|---|
| `core/runtime/modules/log.rs`、`builtin/runtime_modules/core_modules.rs` | Log/LogDiagnostics descriptor和core候选 | descriptor不拥有provider，G04 Fail |
| `dynamic_api/session/{construction,ffi,project,state}.rs`及registry | dynamic lease在DLL映像再次init并按count释放 | lease为Partial基础；单process authority仍Fail |
| `core/runtime/diagnostics/profiling/{mod,macros,tracy}.rs` | bounded profile recorder、可选Tracy global subscriber | profiling底座真实；与process log没有统一Registry/identity |
| `zircon_app/src/bin/{editor,runtime_preview}.rs` | install panic hook、run entry、terminal shutdown | lifecycle意图保留；panic payload与typed terminal缺失 |
| `entry/cli/{diagnostic_log_args,launch_args,mod}.rs` | CLI映射日志配置 | 进入统一validated ProcessLogPolicy compiler |
| `entry/entry_runner/{bootstrap,editor,runtime}.rs` | init/teardown和错误路径 | EarlySpool、single router receipt与shutdown budget缺失 |
| `entry/runtime_library/runtime_session.rs` | DLL session acquire/drop与host teardown | 改持V2 host service lease，禁止DLL process sink |
| 其余Runtime/App实际log/eprintln/tracing消费者 | 100个文件，30,883行 | hard cutover后全部提交统一record；常规fallback不得旁路 |

### 15.3 Editor日志owner与消费者

Editor owner共13个文件：`config.rs`86、`entry.rs`69、`error.rs`43、`filter.rs`32、`jump.rs`67、`mod.rs`26、`record.rs`47、`rolling_file.rs`84、`service.rs`535、`severity.rs`6、`source.rs`110、`store.rs`114、`tests.rs`670。`EditorLogStore`以2,048 records/4MiB默认边界维护sequence；event queue默认256 records/512KiB并能发resync；单entry最大值由store byte cap拒绝。`RollingFileLogSink`按day与32MiB workspace默认切segment，但每次append都在caller同步open/write/flush，没有retention、quota、manifest或独占。

`core/context/{builder,editor_context}.rs`建立默认service和event sink，project open将其配置到`.zircon/logs`；`core/jobs`、script build、settings、scene viewport、`ui/activity`、`ui/host`、retained host和console projection均是真实消费者。这证明Editor不是fixture logger，也证明它确实构成第二套落盘authority。迁移必须保留sequence/store/resync/source/jump/UI能力，删除同步file owner。

### 15.4 Native plugin与发行描述

`native_plugin_loader/{abi_declarations,host_callbacks,native_plugin_abi}.rs`和SDK `native.rs`声明V3 host log/diagnostic函数。production loader entry传入Some callback；callback在`OnceLock<Mutex<BTreeMap<u64, Capture>>>`内按host handle收集owned String，entry返回后被扁平化为descriptor diagnostics。它不是运行期logger，锁覆盖String push，handle是裸递增`u64`，也没有record sequence、budget或sink health。

当前39个`zircon_plugins/*/dist/src/lib.rs`均显式构造`host_log: None`，没有一个使用Some。这些table主要用于distribution/self-validation，不应被误读成production loader没有callback；但它们也说明发行验证没有真实host logging fixture。Runtime132将两者分别记录，避免“全None”或“已闭环”两种过度结论。

## 16. 首个实施切片与完成定义

首个实现切片应停在M0/M1合同和RED证据，不直接堆rotation/UI功能：

1. 写App+Runtime DLL同进程必须共享router generation/global sequence的失败测试。
2. 写`DiagnosticRecordV1`、`LogAdmission`、`InitializeReceipt`、`FenceReceipt`、`SinkHealth` schema/property tests。
3. 写no-output init/shutdown、conflicting init、stale lease、critical timeout clamp、oversize bytes和control fence的失败测试。
4. 定义`ProcessLogHostServiceV2` ownership/lifetime/ABI，不保留DLL内部process sink兼容路径。
5. 只在这些truth tests冻结后进入M2-M8；每个切片独立回写Runtime07 failure和对应gate。

Runtime132只有在52项P1全部Closed、36个Gate全部Pass、Runtime07 failure正式回传fixed、旧global/Editor file/plugin capture旁路删除，并且managed Windows与所需平台证据可按revision/artifact复现时才可标记implementation complete。当前状态保持`review_complete / implementation_pending / source_recheck_required`。
