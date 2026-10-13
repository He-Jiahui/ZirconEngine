---
title: Runtime Failure Contract、Error Taxonomy、Panic Containment、Health、Recovery、Shutdown、Crash、ABI 与 Product Integration 当前源码复核
category: zircon_runtime
report_id: Runtime195
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
canonical_owners:
  - Runtime02
  - Runtime43
  - Runtime44
  - App01
  - Interface01
refreshes:
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-scheduler-scope-cancellation-timer-io-backpressure-shutdown-current-working-tree-review.md
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/runtime_api/abi
  - zircon_runtime_host/src/foreign_output
  - zircon_app/src/entry/runtime_library
  - zircon_app/src/entry/product_shutdown
  - zircon_editor/src/core/gateway/session
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AssertionMacros.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/ThreadHeartBeat.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformCrashContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Launch.cpp
  - dev/bevy/crates/bevy_ecs/src/error/handler.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_app/src/panic_handler.rs
  - dev/godot/core/error/error_macros.h
  - dev/godot/core/error/error_macros.cpp
  - dev/godot/platform/windows/crash_handler_windows_seh.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-core/src/safelock.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphBuilders.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
---

# Runtime195 当前源码审查

## 1. 结论

当前 Zircon 已经有数个可以保留的失败处理底座，但它们仍是彼此断开的局部机制。Core module/service 激活能捕获 factory/lifecycle panic 并尝试回滚；JobScheduler 和 TaskGraph 能把用户任务 panic 转为终态；Scene command queue 和 native system callback 会在继续展开前丢弃未提交 command、归还临时状态；native/VM plugin 热重载能恢复旧 generation；dynamic session 显式销毁失败会保留 slot 和 DLL handle 供重试；App 已有有界 `ProductFailureLedger`、阶段化 `ProductShutdownCoordinator` 和 DLL 卸载前 abort 的 fail-closed 保护。这些实现不是空壳，也不应在重构时被抹掉。

问题在于它们没有共同的 failure contract。`CoreError`、task panic string、plugin `Vec<String>`、VM `Operation(String)`、`ZrStatus`、`RuntimeLibraryError`、foreign-output error、product ledger 和 Editor gateway error 之间反复格式化、压平和重新分类。跨 ABI 后只剩九个 status code 与一段线程本地借用文本；App 又把全部 status 还原为 `General` 字符串。调用者无法稳定判断 error code、failure class、owner/generation、recoverability、retry/backoff、quarantine、user action、source chain、correlation 或 teardown completeness。

panic 策略同样分裂：Core activation 返回 typed error，JobHandle 保存 payload 文本，detached task 只留下通用消息，callback dispatcher 和 wake trampoline 静默吞 panic，terminal observer 只累计次数，Scene/VM 在恢复局部不变量后继续 unwind，FFI 则把任何 panic 压成相同常量。大量 poisoned mutex 直接 `into_inner()` 继续运行，没有验证临界区不变量是否完整。产品层虽然会在无法证明 DLL 已静默时 abort，但没有 crash artifact、emergency-safe failure record、watchdog、hang census、平台异常/OOM 策略或可在进程退出后读取的关停收据。

本轮不新增唯一 P0，Runtime43、Runtime44、Runtime02、App01 与 Interface01 继续拥有各自 session、log/crash、task、process-host 和 ABI 父阻断。Runtime195 新增跨层细化账本为 **46 项 P1（37 Open / 9 Partial）**、**12 项 P2（12 Open）**，资格门 **32 项（25 Fail / 6 Partial / 1 Pass）**。唯一通过的门是 Core module/service activation panic 已有返回错误、回滚及相邻测试；这不能代表 process failure contract 已完成。

本轮只写 review、索引和 coverage，没有修改 production Rust、tests、Cargo、ABI 或产品 UI；没有运行 Cargo、真实 DLL/App/Editor、crash subprocess、Miri、loom、sanitizer、fuzz、fault、scale、soak 或 benchmark。静态审查不能证明性能、稳定性或表现已达到、更不能证明优于 Unreal。

## 2. 审查边界与物理冻结

### 2.1 选择集

| 范围 | 文件 | 主要内容 |
|---|---:|---|
| Core lifecycle、activation、state 与 task failure | 112 | `CoreError`、factory/callback panic、activation rollback、task/job/graph/timer/IO terminal、observer delivery |
| Dynamic ABI、Scene/Script panic 接缝与 native plugin | 71 | FFI panic guard、session registry/destroy、status diagnostics、command invariant cleanup、plugin load/hot reload |
| App product failure 与 shutdown | 30 | runtime library status projection、foreign output、failure ledger、phase coordinator、exit class、fatal DLL unload fence |
| Runtime Interface 与 Host | 22 | `ZrStatusCode`、ABI table/status shape、foreign output kind/policy/state |
| Editor gateway consumer | 16 | gateway/session error、output、protocol 与 session projection |
| 去重 Zircon 选择集 | **251** | **54,976 行 / 1,939,169 bytes / 526 个 test attributes / 38 个 ignored markers** |
| 五引擎参考选择集 | **22** | **18,971 行 / 772,789 bytes / 88 个 test markers** |

Zircon 选择集 fingerprint 为 `6e0f35e615d2871e57cfe7338b08527816327dacc95738ad15af0c922c60b8fc`；参考选择集 fingerprint 为 `3bcea8f593f16333318f9e2def0b1c997ec46f6f9f6ee50534224aa70c7d397b`。算法是相对路径转小写和 `/`，对每个文件计算 SHA-256，按 `path|hash` 排序、LF 连接后再计算 UTF-8 SHA-256。fingerprint 只冻结本轮读取集合，不是发布 artifact 或 ABI identity。

### 2.2 选择规则与未宣称范围

本轮逐层追踪 `domain error -> callback/task boundary -> lifecycle rollback -> session/registry -> FFI status -> App runtime wrapper -> product ledger/shutdown -> Editor gateway`。完整纳入 Core activation/state/task 子树、dynamic session registry、native load report/live host、App runtime library/product shutdown、Host foreign output 与 Editor gateway session；Scene、VM 和 diagnostic log 只纳入直接参与 panic/invariant/crash 接缝的 owner 文件。

词法盘点不是 finding 数。冻结选择集中有 41 次 `catch_unwind`、44 次 `AssertUnwindSafe`、49 次 `panic!`、410 次 `unwrap(`、660 次 `expect(`、65 次 `into_inner()`、5 次 `process::abort`、1 次 `set_hook`、0 次 `watchdog`。排除测试路径后仍有 28/28/18/28/122/65/3/1/0 次。这里的“非测试路径”仍可能包含 inline `#[cfg(test)]`，每个命中也可能有合法不变量依据；本报告只把逐读后有传播或恢复问题的调用列为 finding。

本轮不重新审计 Runtime44/132 已覆盖的完整 log router、rotation、retention 和 durability；`diagnostic_log/sink.rs` 只作为 panic hook 与 emergency evidence 接缝。也不展开 renderer device-loss、resource/asset import、network protocol、audio backend、Editor authoring-domain error、Hub/Coordinator、Tooling failure contract。它们需要各自 owner 的纵向复核，Runtime195 不声称已扫描这些角落。

### 2.3 当前工作树与归属约束

基线是当前 `HEAD 14c89f9776bed828cc85e05e4b9914b3f8d1e784` 加工作树内容。索引、coverage 和相邻 review 已有在途修改，本轮保留并追加，不回退任何他人改动。架构目标继续遵守固定三产品包：`zircon_app` 拥有进程策略，`zircon_runtime` 拥有 runtime failure authority，`zircon_editor` 只做 authoring/UX projection；不恢复历史独立 `zircon_core`/server 包计划。

## 3. 当前失败传播链

### 3.1 Core module/service lifecycle

- `CoreError` 已覆盖 duplicate/missing dependency、factory panic、module callback panic、ready/drain/cleanup timeout、activation rollback 与 batch rollback；这是 typed domain error 的正确方向。
- `handle/activation` 在 activate/deactivate 与 batch cleanup 外包 `catch_unwind`，失败时恢复 startup/reactivation 状态并汇总 cleanup failure；`handle/resolution` 将 service factory panic 映射为 `ServiceFactoryPanicked`。
- 但 `CoreError` 没有稳定 code/class/severity、owner generation、recoverability、retry/restart/quarantine 指令、correlation 或可版本化序列化。跨出 crate 后只能靠 Display 文本识别。
- activation、registry、product coordinator、task state、session store 等锁在 poison 后常直接取回 inner。panic 发生在 mutation 中点时，“还能取得值”不等于 graph、slot、queue 或 phase invariant 已恢复。

### 3.2 Task、callback 与 observer

- scheduled job 用 `catch_unwind` 得到 `JobExecutionOutcome::Panicked(Arc<str>)`，写入 JobHandle、task diagnostics，并阻止依赖任务启动；TaskGraphScope 也会恢复 worker lease并标记失败。
- `JobHandle::wait` 再次 panic 以传播失败，task pool/job scheduler 的普通 `spawn` 在 admission 已关闭时通过 `submission_or_panic` panic；API 没有统一的 fallible admission/terminal receipt。
- detached task 的 Drop guard 只记录 `"detached task panicked"`，原 payload、task identity、scope、dependency chain 和 backtrace 丢失。
- `TaskCallbackDispatcher::CallbackEnvelope::run` 对每个 callback `catch_unwind` 后丢弃结果；bounded keyed IO observer 同样吞掉 panic。Job terminal observer至少有 panic count，但仍没有 callback identity、failure id、owner action或health transition。
- 这些边界不能用一种全局行为替换：dependency continuation 需要终止其 DAG 分支，telemetry callback可隔离并降级，lifecycle callback可能必须终止 session。当前代码没有编译后的 per-boundary policy。

### 3.3 Scene command、system 与 VM/plugin

- external/native Scene system 在 callback panic 后 `discard_pending()` 并继续 unwind；CommandQueue 在 apply panic 时消费剩余命令、reset arenas、清 deferred spawn resolution 后继续 unwind；schedule runner 恢复 flush flag、归还 system 和 command buffer。局部内存与 command invariant 处理是正确底座。
- 但 rethrow 到达哪一层、该 frame/world/session 是否仍可继续、哪些已经执行的 side effect 需要补偿，没有统一 transaction/failure receipt。`AssertUnwindSafe` 只声明捕获安全，不证明 World、system/plugin instance 与 host registration 的语义不变量已恢复。
- VM call export 会在 panic 后把 instance 放回 active slot再继续 unwind；native plugin FFI guard则把 host/plugin/output callback panic统一压成 panic code与静态消息。相同 plugin failure 因入口不同得到相反传播策略。
- native load errors本身有 stage、path、expected/actual、hint/source 等可保留 typed variants，但 `NativePluginLoadReport` 的 diagnostics 是 `Vec<String>`，`has_failures` 以“存在任何 diagnostic”判断，warning、degraded、terminal 与 rollback failure不能稳定区分。
- native/VM hot reload已有旧 generation回滚和 restore failure处理；但 host interface、reflection、registration、external side effect、background work和health/quarantine没有共同 aggregate transaction。rollback failure最终仍被格式化成一条 `Operation(String)`。

### 3.4 Dynamic ABI 与 session teardown

- 所有 exported FFI call通过 panic guard，避免 Rust unwind 穿过 C ABI；`destroy_session_slot` 在 action drain或session shutdown失败时保留 slot供重试，只有成功后才从 registry 移除。这是重要的 fail-closed 基础。
- `ZrStatusCode` 只有 `Ok/Error/UnsupportedVersion/InvalidArgument/NotFound/CapabilityDenied/Panic/BridgeNotEnabled/LimitExceeded`。没有 Busy、Timeout、Cancelled、StaleGeneration、Conflict、Unavailable、DeviceLost、ShutdownIncomplete 或 RetryAfter。
- unknown raw code会降级为 `Error`，新 runtime 对旧 host 的未知状态无法保真；任意内部 error又大多经 `error_status(Display)`压成通用 `Error`。
- diagnostics 存在 4,096-byte线程本地 buffer，边界与 UTF-8 截断有测试，但返回 slice只在同线程下一次 status write前有效；嵌套/reentrant host call可覆盖前一错误。它没有 owned failure id、schema、source chain、context或恢复动作。
- FFI panic只返回常量文本；`get_api` panic直接返回 null。callback名、plugin/session generation、panic payload/backtrace、是否隔离、后续health均不可查询。
- `RuntimeDynamicSession::shutdown_before_library_unload` 返回单个 bool，逐步关闭 event mirror、watchers、modules、task graph与process log；无法表达哪些 owner完成、哪些未完成、是否可重试。其 `Drop` 直接 `let _ = ...` 丢弃 false。
- session destroy的 module drain timeout当前为零，而外层 action drain和其他 shutdown path又有不同策略；一份 bool无法证明所有 callback、allocation、job、watch、subscription、surface与DLL代码都已静默。

### 3.5 App、Product 与 Editor consumer

- App `RuntimeSession::try_destroy` 在 ABI失败时保留 runtime library和handle，允许显式重试；Drop失败时记录 teardown ledger并 `abort()`，防止仍活跃的 DLL callback在库卸载后执行。安全方向正确。
- `ensure_status` 把九种 ABI code转为字符串，再创建默认 `RuntimeLibraryError`；`RuntimeLibraryErrorKind`只有 General/CapabilityUnavailable/ProtocolViolation，故 Panic、InvalidArgument、NotFound、LimitExceeded与普通 Error跨层后再次合并。
- foreign output也只有 RuntimeCall/ProtocolViolation 两类，无法表达 allocation leak、owner mismatch、stale generation、partial release或host rejection的恢复策略。
- `ProductFailureLedger` 有顺序、phase、severity、owner、16-record上限、512-byte UTF-8安全转义/截断与 suppressed count，是可保留的冷路径底座；但先到的16条 recoverable failure会挤掉后续 emergency/teardown，且没有 failure id、code、generation、time、source chain、dedup、severity reserve或durable artifact。
- `ProductShutdownCoordinator` 记录首个 terminal reason、合法phase迁移、disposition与elapsed；但没有 expected-owner census、per-owner terminal receipt、deadline/budget、late callback fence或durability。`NoOwner`/`LegacyCombined`可以在没有执行独立owner teardown时满足phase结构。
- semantic `ProductExitClass` 已区分 startup/runtime/shutdown/forced，但全部失败最终映射为进程码1；外部supervisor、CI和crash triage无法据code区分退出阶段。
- fatal Drop路径在 ledger记录后只 `eprintln!` 再 abort。进程内 ledger没有机会被正常consumer读取，也没有预打开 emergency handle、预分配crash context、abort reason、module/thread/task census或support bundle。
- Editor gateway接收的仍是域内 error或格式化 runtime error；没有 failure-id查询、typed recovery action、retry/backoff/quarantine状态、source/owner导航或“继续/重启session/禁用plugin/重启进程”的可信UX合同。

## 4. 已有 owner 与 P0 路由

本篇新增 **0 项唯一 P0**，也不把同一根问题换名重复计数：

| Canonical owner | 已有根合同 | Runtime195责任 |
|---|---|---|
| Runtime02/192 | task admission、cancellation、scope、drain、shutdown | 统一task panic/failure envelope、supervisor decision和health publication，不重复task scheduler finding |
| Runtime43 | dynamic session registry、ABI action、allocation、destroy与DLL unload | 定义typed failure/status/shutdown receipt的纵向传播，不重复session并发与feature功能finding |
| Runtime44/132 | process log、panic hook、crash durability | failure authority向router/emergency artifact提供结构化记录；不重复52项日志finding |
| App01 | process bootstrap、loop、shutdown与最终退出 | App拥有process crash policy、watchdog、artifact与numeric exit；Runtime不得成为第二进程owner |
| Interface01 | DLL ABI、version、handle与foreign ownership | 扩展稳定failure/status ABI、unknown保真和owned query；不在Rust私有enum上假装ABI完成 |
| Plugins01/Runtime07 | plugin ABI、execution、reload与host callback | plugin generation、quarantine和aggregate rollback接入统一failure contract |
| Editor gateway/notification owners | Editor session与UX projection | Editor只消费typed failure和action，不决定Runtime恢复事实 |

Runtime44 的 `R44-P1-39/40`（panic payload/emergency writer与abort/SEH/signal/OOM）及 Runtime43 的 `DYN-P1-006/011`（混合shutdown等待与Drop丢结果）保持原owner。Runtime195只补齐它们与Core/task/plugin/ABI/App之间缺失的共同合同。

## 5. P1 差距与重构内容

| ID | 状态 | 当前差距 | 需要重构的内容 |
|---|---|---|---|
| `RFC-P1-01` | Open | 没有跨Core/task/plugin/session/App的稳定failure vocabulary | 定义versioned `FailureCode`、`FailureClass`、`FailureSeverity`和domain registry；domain typed error继续保留 |
| `RFC-P1-02` | Partial | Core/native load等有typed error，但跨层立即Display化 | 每个边界实现显式adapter，保留domain variant、source与字段；禁止通用`to_string -> General` |
| `RFC-P1-03` | Open | failure没有稳定identity/correlation | 生成`FailureId`并绑定process/session/world/module/plugin/task/operation/frame与provider generation |
| `RFC-P1-04` | Open | source chain和附加context只能拼字符串 | 用bounded typed fields与source references编码，formatter只是sink/UX projection |
| `RFC-P1-05` | Open | severity不等于recoverability，调用者没有可靠动作 | 独立`RecoveryDirective::{Continue,Retry,Backoff,RestartScope,Quarantine,Shutdown,Abort}`与执行receipt |
| `RFC-P1-06` | Open | error字段无敏感信息分类、redaction和导出策略 | schema声明visibility/privacy，日志、Editor、ABI、crash bundle按principal与sink policy投影 |
| `RFC-P1-07` | Open | health只有零散计数/状态，没有owner state machine | 建立generation-bound `HealthState::{Healthy,Degraded,Failed,Quarantined,Stopping,Terminal}`及合法迁移 |
| `RFC-P1-08` | Partial | task diagnostics、product ledger等能记局部失败，但无single authority | Runtime建立bounded `FailureAuthority` journal/query/subscription；App只拥有process terminal与artifact policy |
| `RFC-P1-09` | Open | 重复失败可形成风暴，没有budget/backoff/circuit breaker | 按code+owner+generation聚合，配置rate/first-last/suppressed；恢复失败触发circuit breaker/quarantine |
| `RFC-P1-10` | Open | 新error variant/status/boundary无需登记映射或测试 | 编译failure manifest并lint所有public Result、panic boundary、ABI adapter、recovery consumer |
| `RFC-P1-11` | Partial | 已有多处catch/rollback，但各入口政策不一致 | 注册`FailureBoundaryPolicy`：可捕获payload、所需postcondition、传播目标、是否允许继续与terminal action |
| `RFC-P1-12` | Open | `AssertUnwindSafe`后没有统一invariant proof | 每个boundary实现显式cleanup/validation receipt；验证失败立即升级scope/session/process，不继续假健康 |
| `RFC-P1-13` | Open | 多数panic只剩常量或短字符串 | 冷路径提取bounded payload、callsite、thread/task、owner、backtrace disposition并分配FailureId |
| `RFC-P1-14` | Open | callback dispatcher吞panic | callback envelope携带identity/owner/policy；dispatcher发布terminal callback result并按策略隔离或升级 |
| `RFC-P1-15` | Open | runtime wake trampoline panic静默丢失 | wake registration返回/记录callback failure，禁用该generation并唤醒supervisor，避免永久假注册 |
| `RFC-P1-16` | Open | terminal/IO observer最多只计数或完全吞panic | observer delivery返回receipt；panic计入owner health，shutdown census证明accepted observer已终结 |
| `RFC-P1-17` | Open | poisoned control locks普遍`into_inner`后继续 | 将poison变成typed invariant failure；只允许经owner-specific validate/repair，否则fail-close并退休generation |
| `RFC-P1-18` | Open | failure handler自身递归/panic没有one-shot保护 | 加thread-local reentrancy guard和最小stderr/emergency fallback；handler-origin panic不能被再次包装成普通error |
| `RFC-P1-19` | Open | TaskPool/JobScheduler关闭admission后panic | public submission返回typed `Closed/Overloaded/Stopping`；只在已证明的内部不变量使用panic |
| `RFC-P1-20` | Open | detached task只报告通用panic文本 | detached任务也必须属于scope/supervisor，保留task identity、payload、owner、terminal time和join/reap证据 |
| `RFC-P1-21` | Partial | Job/TaskGraph有Panicked终态和依赖阻断 | 终态增加failure id、causal dependency、cancellation race、recovery decision与observable result store |
| `RFC-P1-22` | Open | Scene/VM cleanup后rethrow，但产品不知道world/session是否可继续 | 为system/export/frame boundary定义continue/restart-world/restart-session/abort policy和postcondition |
| `RFC-P1-23` | Partial | CommandQueue能discard/reset，schedule能归还buffer/system | 输出`SceneExecutionFailureReceipt`，列出已提交、已补偿、已丢弃side effect与world health generation |
| `RFC-P1-24` | Open | VM call、native FFI、Scene extension对同类plugin panic处理相反 | 收敛统一PluginExecutionBoundary；ABI内不unwind，进程内按manifest policy传播，结果都进入同一failure id |
| `RFC-P1-25` | Open | native load report用`Vec<String>`且任意diagnostic即failure | 使用typed diagnostic severity/code/stage/owner，分别计算warning/degraded/rejected/rollback-failed disposition |
| `RFC-P1-26` | Partial | native/VM hot reload可回滚旧generation | 用aggregate transaction覆盖registrations/reflection/jobs/callbacks/external effects，输出逐参与者commit/rollback receipt |
| `RFC-P1-27` | Open | plugin failure不驱动generation quarantine/circuit breaker | 超预算panic/protocol/recovery failure禁止新call，drain lease，隔离generation并向Editor暴露可信状态 |
| `RFC-P1-28` | Open | ABI九种status无法表达运行时失败空间 | 增加稳定分类或`ZrFailureV2`，覆盖busy/timeout/cancel/stale/conflict/unavailable/device-lost/shutdown-incomplete |
| `RFC-P1-29` | Open | unknown raw status被改写为`Error` | status wrapper保留raw code与known/unknown view；旧consumer必须能转发未知值而不伪造已知语义 |
| `RFC-P1-30` | Open | diagnostics是同线程下一次调用前有效的borrowed buffer | 改为host-provided bounded buffer、owned result或failure-id query；定义reentrancy、释放与截断receipt |
| `RFC-P1-31` | Open | `error_status(Display)`丢失domain variant与恢复信息 | 每个domain到ABI有显式映射表；unmapped variant在CI失败，禁止静默通用Error fallback |
| `RFC-P1-32` | Open | FFI panic恒定消息，`get_api` panic只返回null | API negotiation也提供可查询bootstrap failure；panic status带boundary/owner/generation和artifact correlation |
| `RFC-P1-33` | Open | ABI没有稳定failure query与schema negotiation | 提供versioned bounded `get_failure/release_failure`或host buffer API，并报告supported schema/capabilities |
| `RFC-P1-34` | Open | `RuntimeDynamicSession::Drop`丢弃shutdown false | 所有正常owner走显式close；Drop只做已记录的last-resort隔离，失败必须进入process terminal path |
| `RFC-P1-35` | Open | session shutdown只返回bool | 返回逐owner `SessionShutdownReceipt`：admission/actions/callbacks/jobs/modules/watches/allocations/surfaces/log leases |
| `RFC-P1-36` | Open | module零等待与外层不同等待策略没有共享预算 | App创建总deadline，按phase/owner分配budget并记录timeout、abandon/quarantine和剩余census |
| `RFC-P1-37` | Partial | ProductFailureLedger有bounded顺序、severity、owner和suppressed count | 升级为typed failure reference；为terminal/emergency预留容量并支持dedup、source、generation与durable handoff |
| `RFC-P1-38` | Open | first-16策略可让早期recoverable错误遮蔽teardown emergency | 分级ring/reserve与summary slots；任何丢弃都有按severity/code/owner统计且最终写入artifact manifest |
| `RFC-P1-39` | Partial | ProductShutdownCoordinator有phase、首因、disposition与耗时 | 增加expected owner set、phase deadline、per-owner receipt、late work census、completeness与durability等级 |
| `RFC-P1-40` | Open | `NoOwner/LegacyCombined`可在未执行独立cleanup时完成phase | disposition只描述兼容路径，不能满足required owner；缺owner必须Degraded/Failed并有退出预算 |
| `RFC-P1-41` | Open | 四种failure exit class全部映射code 1 | 冻结平台可移植numeric mapping并允许supervisor读取artifact id；不要用大量任意业务exit code |
| `RFC-P1-42` | Partial | teardown失败abort能避免unsafe DLL unload | abort前走allocation-free emergency writer、冻结crash context/census并持久化failure id；不得只eprintln |
| `RFC-P1-43` | Open | 无hang watchdog、heartbeat、deadlock/long-stall终端策略 | App/platform拥有watchdog，线程/scope注册heartbeat与suspend token，超时生成hang artifact后按policy终止 |
| `RFC-P1-44` | Open | Rust panic hook不覆盖abort/SEH/signal/OOM/GPU hang | 建立平台CrashArtifactCoordinator；每平台声明可捕获范围、安全操作和external collector交接 |
| `RFC-P1-45` | Open | App/Host/Editor error kind过少且UI只得到字符串 | 保留typed category/failure id/recovery action；Editor可导航owner、重试、禁用plugin、重启session或只读恢复 |
| `RFC-P1-46` | Open | 没有跨boundary的系统故障注入与资格矩阵 | 建立unit/property/concurrency/process/crash/fault/scale/soak七层测试，并归档machine-readable receipts |

## 6. P2 性能、诊断与维护差距

| ID | 状态 | 差距与目标 |
|---|---|---|
| `RFC-P2-01` | Open | 热路径error code/context使用interned/static id，禁止每次格式化大String后才分类 |
| `RFC-P2-02` | Open | `FailureEnvelope`采用small/bounded字段和外部blob引用，不让罕见失败类型膨胀所有task/operation handle |
| `RFC-P2-03` | Open | source chain、backtrace、attachments有字节/深度/时间预算并返回truncated disposition |
| `RFC-P2-04` | Open | failure journal与emergency context预分配、避免crash path拿普通owner锁；实现必须以测量决定ring/shard布局 |
| `RFC-P2-05` | Open | 重复error按window聚合first/last/count/sample，不在日志、Editor和telemetry各自重复风暴控制 |
| `RFC-P2-06` | Open | backtrace按severity/first-occurrence/sample策略采集，记录capture cost与disabled/unavailable原因 |
| `RFC-P2-07` | Open | failure budget按owner/domain/session隔离，单个坏plugin不能耗尽全进程journal或callback lane |
| `RFC-P2-08` | Open | health/failure snapshot需要generation/window/completeness/overflow，计数saturation必须显式terminal flag |
| `RFC-P2-09` | Open | numeric exit、failure code、artifact schema、Editor localization分别版本化，避免文案成为协议 |
| `RFC-P2-10` | Open | support bundle关联logs/metrics/crash/session shutdown receipt，执行大小、隐私、retention和加密策略 |
| `RFC-P2-11` | Open | 测量steady-state零失败开销及panic/error storm p50/p95/p99/max、allocation、RSS、lock wait、shutdown time |
| `RFC-P2-12` | Open | 生成failure registry文档/ABI header/test vectors；lint失去owner、mapping、recovery consumer或qualification的条目 |

## 7. 参考引擎证据与适用边界

| 参考 | 可吸收的工程证据 | 不能照搬 |
|---|---|---|
| Unreal Engine | `ensure`与fatal `check`分层，ensure可继续并保留report/stack；ThreadHeartbeat有hang owner与回调；CrashContext有context type、唯一identity、attachments和预留buffer；Launch有process guarded boundary | 宏、全局singleton、平台历史兼容与C++异常策略不适合作为Rust domain error API；Zircon仍需稳定DLL ABI和typed receipt |
| Bevy | `ErrorContext`区分System/RunCondition/Command/Observer并带name/tick；handler按severity决定ignore/log/panic；thread-local标记防止handler自身panic被递归包装；executor测试验证handler-origin panic重抛 | Bevy主要是单进程Rust ECS，不提供Zircon的跨DLL owned failure、process crash artifact或session quarantine |
| Godot | ERR/CRASH宏明确early-return与fatal；error handler去重，thread-local reentrancy guard在递归、OS未初始化/已销毁时直接stderr；Windows SEH handler通知MainLoop并输出stack | Variant/宏/全局handler不能替代typed Rust error和owner generation；SEH stack print也不是完整minidump/durable receipt |
| Fyrox | GameErrorSource保留plugin/script method与scene/node context，集中`on_game_error`允许plugin消费；SafeLock用明确timeout避免永久挂死并有测试 | 一条error queue和10秒panic锁不是完整supervisor/watchdog；不能直接作为ABI或恢复政策 |
| Unity Graphics | RenderGraph builder `finally`恢复command-buffer mode；执行异常后reset graph state、cleanup resource并clear command buffer；resource pool在exception时回收frame allocation并有Editor tests | 这是RenderGraph局部invariant cleanup，不拥有process crash、plugin/session health或跨DLL failure taxonomy；只作为事务清理下限 |

共同启示不是“复制某引擎的错误系统”，而是分开四件事：可恢复domain error、违反不变量、可隔离panic、进程级crash/hang。Zircon若要超过参考实现，还必须增加跨DLL保真、generation-bound identity、可执行RecoveryDirective、逐owner teardown receipt、fault budget与可归档资格证据。

## 8. 目标架构与所有权

### 8.1 不是万能错误枚举

`CoreError`、`VmError`、`PluginLoadError`、render/resource/domain error继续是各自模块内的typed Rust enum。统一层只携带跨域所需的稳定中立字段，不吸收每个domain variant：

```text
Domain Result<T, DomainError>
  -> explicit BoundaryAdapter
  -> FailureEnvelope
       id / code / class / severity
       owner + generation + operation
       bounded context + source references
       recoverability + RecoveryDirective
       privacy + artifact correlation
  -> FailureAuthority
       journal / aggregation / health / policy / subscription
```

### 8.2 包与模块owner

| Owner | 必须拥有 | 禁止拥有 |
|---|---|---|
| `zircon_runtime::core::framework::failure` | neutral code/class/severity/context/directive/envelope DTO与domain adapter traits | process abort、Editor文案、所有domain variant的大enum |
| `zircon_runtime::core::runtime` | per-runtime/session FailureAuthority、health state、policy、journal、query/subscription与module/task/plugin adapters | OS crash handler、最终numeric exit |
| `zircon_runtime::core::manager` | 稳定handle、snapshot/query与owner-facing recovery request | 私自修改health或吞failure |
| `zircon_runtime_interface` | versioned `ZrFailureV2`/query、raw status保真、owned/buffer lifetime、layout tests | Rust trait object、借用线程本地错误作为长期ABI |
| `zircon_runtime_host` | foreign output/protocol failure适配与budget enforcement | 重新分类为无差别String |
| `zircon_app` | ProcessFailurePolicy、CrashArtifactCoordinator、watchdog、shutdown budget、最终exit/abort | 第二套runtime domain truth或Editor UX状态 |
| `zircon_editor` | typed failure projection、localization、owner导航、recovery command UX与history | 猜测Runtime是否可继续、以toast关闭failure |

### 8.3 恢复与终止流程

```text
callback/task/FFI/domain failure
  -> boundary captures + restores local invariant
  -> validate postcondition
  -> publish FailureEnvelope
  -> FailureAuthority updates owner HealthState
  -> policy emits RecoveryDirective
       Continue / Retry / Backoff / RestartScope / Quarantine
       ShutdownSession / ShutdownProcess / AbortProcess
  -> executor returns RecoveryReceipt
  -> App shutdown/crash owner persists terminal artifact
  -> Editor/supervisor consumes typed projection
```

panic捕获绝不自动等于可以继续。只有 cleanup成功、postcondition通过、owner policy允许且recovery receipt提交后，scope才可恢复Healthy/Degraded。否则必须退休对应generation，避免已经发生panic的plugin/world/session继续对外宣称Ready。

### 8.4 ABI候选

```text
ZrStatusV2 {
  raw_code: u32,
  flags: u32,
  failure_id: ZrFailureId,
  retry_after_ns: u64,
}

get_failure_v2(session, failure_id, host_buffer) -> ZrWriteReceipt
release_failure_v2(session, failure_id) -> ZrStatusV2
```

具体layout要由Interface owner完成ABI设计与历史consumer matrix；本篇只冻结要求：unknown raw保真、诊断有owner或host buffer生命周期、大小可协商、failure id稳定、截断可见、panic/bootstrap失败可查询、旧ABI有明确hard-cutover或兼容预算。

## 9. 分层实施计划

### M0：Owner与baseline收敛

- 复核Runtime02/43/44/132、App01、Interface01、Plugins01的开放finding，生成唯一owner/dependency manifest。
- 把本轮251文件选择规则固化为可复跑review inventory；源码变化后重算fingerprint。
- 冻结“domain error / invariant violation / panic / crash-hang”术语与非目标。

### M1：Failure schema与ABI保真

- 先写code registry、unknown raw roundtrip、bounded context、source chain、privacy与owned ABI失败测试。
- 实现neutral envelope和逐domain adapter，禁止默认Display fallback。
- App/Host/Editor先保留typed code/failure id，再移除旧字符串猜测。

### M2：统一panic boundary

- inventory所有catch/unwind/callback/observer/FFI边界并绑定policy、owner和postcondition。
- callback dispatcher、wake、terminal/IO observer不再静默吞panic。
- task admission改为fallible；detached work必须归scope/supervisor。

### M3：不变量、poison与transaction receipt

- Core/Scene/VM/plugin为cleanup定义可验证postcondition与rollback receipt。
- poison按owner验证/repair或retire generation，删除无条件`into_inner`成功路径。
- plugin hot reload扩展为跨registration/reflection/job/callback的aggregate transaction。

### M4：Health、隔离与恢复

- 实现FailureAuthority、HealthState、failure budget、aggregation、backoff和circuit breaker。
- plugin/module/task/world/session发布generation-bound health；恢复动作有executor与receipt。
- Editor消费相同状态，不创建平行truth。

### M5：Shutdown与crash artifact

- bool shutdown升级逐owner receipt，App分配总deadline并保留未排空census。
- fatal DLL teardown在abort前写入allocation-free emergency artifact。
- 接入panic/abort/Windows SEH/Linux signal/macOS exception/OOM策略与外部collector边界。

### M6：Product与Editor闭环

- Product ledger升级typed/durable handoff；phase coordinator验证required owner和late work。
- 冻结exit class/code/supervisor协议；Editor实现retry、quarantine、restart session与support bundle UX。
- 移除`NoOwner/LegacyCombined`对required owner的伪完成路径。

### M7：资格与性能

- 运行unit/property/loom/Miri/fuzz、panic/error storm、poison/reentry、DLL skew、crash subprocess与平台matrix。
- 做1/64/1K owner、1M failures聚合、hung callback/worker、disk-full、OOM policy、100h soak。
- 记录零失败热路径开销与故障路径p99/RSS/shutdown预算；达不到预算不得升级Ready。

## 10. 资格门

| Gate | 状态 | 验收条件 |
|---|---|---|
| `RFC-G01` | Partial | domain typed error保留且每个跨域边界有显式mapping manifest |
| `RFC-G02` | Fail | stable failure code/class/severity在Core/task/plugin/session/App一致 |
| `RFC-G03` | Fail | unknown ABI status raw roundtrip不降级为通用Error |
| `RFC-G04` | Fail | ABI failure diagnostics有owned/host-buffer生命周期与释放receipt |
| `RFC-G05` | Fail | failure绑定owner/session/world/plugin/task/operation generation与correlation |
| `RFC-G06` | Fail | recoverability和RecoveryDirective有执行者、deadline与terminal receipt |
| `RFC-G07` | Fail | sensitive fields按sink/principal做redaction，support bundle通过privacy测试 |
| `RFC-G08` | Fail | 所有panic/callback/observer/FFI boundary登记policy与postcondition |
| `RFC-G09` | Pass | Core service/module activation panic返回typed error并执行rollback相邻测试 |
| `RFC-G10` | Partial | Job/TaskGraph panic成为终态并阻断依赖，但仍需failure id/causal receipt |
| `RFC-G11` | Fail | callback/wake/observer panic不再静默，只按声明政策隔离或升级 |
| `RFC-G12` | Partial | Scene command/arena/system在panic后恢复局部不变量，但需world health receipt |
| `RFC-G13` | Partial | plugin FFI不让unwind越过ABI，但需identity/payload/quarantine与health |
| `RFC-G14` | Fail | plugin failure budget触发generation quarantine并能有界drain/recover |
| `RFC-G15` | Fail | poisoned control state必须validate/repair或fail-close，无裸`into_inner`成功 |
| `RFC-G16` | Fail | failure handler递归/panic走one-shot allocation-bounded emergency path |
| `RFC-G17` | Fail | session shutdown返回逐owner completeness、deadline、census和durability |
| `RFC-G18` | Partial | explicit destroy失败保留slot/library供重试，并需升级typed receipt |
| `RFC-G19` | Fail | 正常Drop不再丢弃teardown失败或依赖不可消费的bool |
| `RFC-G20` | Partial | Product ledger有bounded/order/severity基础，并需typed/durable/emergency reserve |
| `RFC-G21` | Fail | phase只在required owner receipt齐全后完成，Legacy/NoOwner不得伪绿 |
| `RFC-G22` | Fail | shutdown共享总deadline，watchdog能报告hung owner与late callback |
| `RFC-G23` | Fail | panic/abort artifact含failure id、payload disposition、thread/task/module census |
| `RFC-G24` | Fail | Windows/Linux/macOS的panic/SEH/signal/OOM/hang矩阵有真实process evidence |
| `RFC-G25` | Fail | semantic exit class映射稳定numeric code与artifact correlation |
| `RFC-G26` | Fail | Editor显示typed owner/code/action且不会从文案猜recoverability |
| `RFC-G27` | Fail | retry/backoff/restart/quarantine动作幂等、可取消并有result receipt |
| `RFC-G28` | Fail | factory/task/callback/plugin/ABI/shutdown每类都有确定性fault injection |
| `RFC-G29` | Fail | panic/abort/DLL teardown/hang使用subprocess测试验证artifact和exit |
| `RFC-G30` | Fail | Miri/loom/sanitizer/fuzz覆盖foreign lifetime、poison、reentry和race |
| `RFC-G31` | Fail | error storm、hung owner、1M failure、100h soak满足RSS/latency/conservation预算 |
| `RFC-G32` | Fail | 对照五引擎的恢复/crash/cleanup能力形成可归档acceptance matrix |

## 11. 反临时实现规则

1. 不用一个万能`EngineError`吸收全部domain error；统一的是跨边界contract，不是业务variant。
2. 不以`catch_unwind`存在宣称panic安全；必须证明cleanup、postcondition、health transition和terminal action。
3. 不以`eprintln!`、一段String、panic count或log line代替failure receipt。
4. 不把poisoned mutex的inner值视为自动可用；mutation中点可能已经破坏graph/slot/queue invariant。
5. 不让FFI返回借用线程本地diagnostics后再发生可覆盖它的reentrant call。
6. 不用`bool`表示多owner shutdown；每个owner必须有identity、deadline、terminal disposition与census。
7. 不因fail-closed abort方向正确就跳过crash artifact、emergency-safe path与process test。
8. 不让Editor根据文本决定retry、continue或restart；动作来自Runtime/App权威并返回receipt。
9. 不把普通可恢复错误、invariant violation、panic和process crash混成同一severity或exit code。
10. 不用单元测试通过替代真实DLL、跨版本、故障注入、subprocess crash、平台和长期soak资格。

## 12. 当前完成定义

Runtime195只有在以下条件同时满足后，才能从`review_complete / product_incomplete`升级：

- 46项P1和12项P2均由唯一owner接收，Open条目有dependency-ordered milestone；
- 32个gate有可复跑命令、环境manifest和artifact，不以source guard替代行为；
- domain errors跨Core/task/plugin/session/ABI/App/Editor保留stable code、identity、source与recovery；
- 所有panic/poison/callback边界有postcondition、health与terminal policy；
- session/product shutdown提供逐owner receipt，DLL unload前能证明静默或进入可取证abort；
- panic/abort/SEH/signal/OOM/hang在支持平台有明确覆盖与真实process evidence；
- failure storm与正常零失败路径满足冻结的CPU、allocation、RSS、latency和shutdown预算；
- Runtime43/44/132、App01、Interface01等canonical owner更新状态，且没有用Runtime195重复关闭其P0。

在此之前，当前实现应描述为“具备若干正确局部隔离与回滚基础，但缺工程级统一失败、恢复、健康、关停和崩溃闭环”，不能描述为已经达到或优于Unreal级可靠性。
