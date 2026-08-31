---
title: Runtime Operation Service / Admission / Prepare / Apply / Cancel / Shutdown 当前源码复审
category: zircon_runtime
report_id: Runtime130
review_date: 2026-08-24
baseline_head: 0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1
baseline_epoch: 383
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
related_code:
  - zircon_runtime/src/operation
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/registry
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/navigation/operation
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_editor/src/core/gateway
  - zircon_plugins/navigation/editor/src/operation_command
  - zircon_plugins/navigation/runtime/src/manager.rs
tests:
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/operation/tests/inflight_retention.rs
  - zircon_runtime/src/operation/tests/phase_indexes.rs
  - zircon_runtime/src/operation/tests/source_guards.rs
  - zircon_runtime/src/operation/service/completion.rs
  - zircon_runtime/src/dynamic_api/tests/operation.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/navigation/runtime/src/tests/operation.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/41/2026-08-19-queued-snapshot-index.md
  - docs/plans/optimize/zircon_runtime/41/2026-08-19-ready-apply-index.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/IAssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/AssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetCompilingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/QueuedThreadPool.h
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/PathTracing/LightBakerWorkerProcessImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water/AsyncTextureSynchronizer.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime130 · Operation Service / Admission / Prepare / Apply / Cancel / Shutdown 当前源码复审

## 1. 结论

`RuntimeOperationService`已经不再是最初的同步轮询器，也不是空壳。当前源码具备session-local handler registry、raw request与retained-byte admission、owner snapshot、worker prepare、owner apply、panic转terminal、deadline/TTL maintenance、cancel publication linearization、两阶段foreign harvest，以及固定48-byte、无分配的poll status。2026-08-19落地的`queued_snapshot_tasks`与`ready_apply_tasks`两个`VecDeque`还消除了两处`HashMap`随机扫描；当前未提交工作树进一步保证cancelled/expired但仍在prepare的task不会在worker完成前被pressure eviction，并为deadline、TTL和completion线性扫描增加profile counter。

这些进展值得保留，但它们仍没有把服务提升为可承载Bake、Cook、Import、World Build和服务器维护的Operation Control Plane。handler registry仍只有`String -> Arc<dyn RuntimeOperationHandler>`；request仍只有operation ID和JSON；handle仍是session内裸`u64`；snapshot仍拿到`world_mut()`；prepare仍没有task lease、cancel token、deadline、progress或budget context；apply仍只返回`Result<()>`，无法证明失败前是否已修改World、driver或外部系统。两个FIFO只解决当前单一队列的选择复杂度：ready队列按多worker完成到达顺序入队，128项顺序测试又把in-flight限制为1，尚未证明并行prepare、priority、fairness、resource conflict或replay determinism。

产品链路也没有闭合。动态V7表仍只有submit/poll/harvest；submit经`with_session`执行而不请求`RuntimeFrameActivity` wake；`RuntimeDynamicSession::frame_demand()`只观察asset reload与animation；Session销毁在module shutdown前后都没有Operation close/cancel/drain/fence。唯一生产handler family仍是四个navigation operation，其中Bake Scene/Bake Surface的prepare固定返回“requires a pure prepare backend”；Editor command仍在调用线程上`yield_now`轮询16次；in-process gateway明确返回capability missing；navigation runtime测试仍调用已不存在的`poll(context, handle)`并期待Bake成功。

Runtime41的48项P1本轮按当前源码重判为 **30 Open、18 Partial、0 Closed**；12项P2为 **8 Open、4 Partial、0 Closed**；40项资格门为 **30 Fail、8 Partial、2 Pass**。Partial只表示一个局部机制真实存在，不能表述为产品能力已完成。本文不新增P0/P1/P2，不复制Runtime02、Runtime24、Interface01/05、Editor09/19或Runtime08D的canonical ownership。

## 2. 审查边界、方法与currentness

### 2.1 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations / ignored | 本轮证据 |
|---|---:|---|
| Operation core与direct tests | **13 / 3,215 / 3,004 / 114,811 / 29 / 1** | registry、admission、task、FIFO、completion、maintenance、harvest与当前在途保留修复；fingerprint `31795ab4685abfd5aa5c2577be0127ec7b4279cdd64298df0f038dbc9b2bb064` |
| ABI、dynamic session、App与Editor gateway | **22 / 5,363 / 4,825 / 188,174 / 19 / 0** | V7 table、bounded JSON、foreign output、frame demand/wake、session teardown与三类gateway；fingerprint `446fbb7c67e03ff2202d75032aa780041aa5fa56f056ac4d502d9ec1c2b5eb68` |
| Navigation producer/consumer | **12 / 2,078 / 1,895 / 71,770 / 9 / 0** | 四个handler、builtin/plugin runtime、Editor command和漂移测试；fingerprint `a8ed929caab3e1178d79821b80f94dc31cf34a6c49bc0aae24fccd383c8677b9` |
| Zircon去重聚焦集 | **47 / 10,656 / 9,724 / 374,755 / 57 / 1** | 上述三组按normalized path去重；fingerprint `820695978c05e05f85032942a273924064c44bc15df1d3bf11478b37627eaae8` |
| canonical owner与实施记录 | **13 / 5,693 / 3,986 / 584,932 / 4 / 0** | P0路由、task/identity/ABI/job/transaction父合同与两个FIFO记录；fingerprint `99d0b6566bd0a57a9bfe755084c97c252fe67996265e4cba8b8b9c9c7ce1b573` |
| 五引擎参考集 | **13 / 4,988 / 4,325 / 173,842 / 7 / 0** | Unreal 5、Godot 2、Bevy 2、Fyrox 2、Unity Graphics 2；fingerprint `6d1cb696e6a28a79d52d88f76cb35c40a0f49a31cb1da1097e082a2b3cd0ce56` |

指纹算法为：每个normalized relative path对应文件lowercase SHA-256，按path排序，以`path|hash`和LF连接且无末尾LF，再取整体SHA-256。测试数字是静态declaration计数，不表示已编译或通过。Operation core指纹包含当前工作树中的未提交改动；基线提交仍为`0e2bdaa9...`，后续实施或验收必须重取source hash，不能把本报告当成这些改动已经集成的证明。

### 2.2 检查方法

本轮沿`register -> submit -> encoded admission -> decode -> queue claim -> owner snapshot -> worker dispatch -> prepare completion -> owner apply -> terminal publication -> poll/harvest -> TTL/pressure eviction -> session close`逐段阅读，并反向检索全部production `register_handler`和operation常量consumer。每段分别核对identity、owner、linearization、capacity、memory residency、cancel/deadline、effect truth、wake、retention、teardown与product reachability；只有source、consumer、behavior test和release receipt闭合才允许Closed或Pass。

### 2.3 动态证据边界

本轮为review-only，只修改报告与索引。没有执行Cargo、当前未提交Operation测试、真实Editor/Nav Bake、reactive host、DLL unload、timer failure、1000任务并发、result oversize、race/model、soak或同硬件同负载benchmark。两个FIFO记录明确写明combined managed Cargo仍pending；ignored release benchmark和当前untracked inflight test均不能被本报告升级为绿色资格证据。

## 3. 必须保留的工程基础

1. 保留每session独立的service与construction-time builtin registration，不退回进程全局任务map。
2. 保留owner snapshot、worker prepare、owner apply三段方向，但以只读snapshot context和typed commit receipt收紧能力。
3. 保留count/byte admission、checked arithmetic和prepared command/result在apply前预留，统一FFI与service reservation owner。
4. 保留固定布局、allocation-free poll status；progress语义必须重做，不能退回每poll JSON和String分配。
5. 保留panic containment、WorkerChannelLost terminalization和prepare slot census，将其接入service fuse与owned task lease。
6. 保留deadline与terminal TTL分离、单service maintenance alarm和最早deadline重臂方向，但删除静默refresh失败与wrapping generation。
7. 保留cancel在apply claim前阻止publication、当前`prepare_in_flight`非驱逐约束，再增加cooperative stop和TooLate receipt。
8. 保留`prepare_harvest -> foreign allocation -> commit/rollback_harvest`，把caller identity、result generation和paged/artifact result补齐。
9. 保留navigation clear/restore的compare-before-replace，提升为descriptor conflict key、generation和commit disposition通用合同。
10. 保留Runtime02 scheduler、Runtime24 identity、Interface01/05 ABI/foreign allocation和Editor09 job projection的唯一authority，不在Operation内部复制第二套平台。

## 4. 当前链路事实与断路

| 链路 | 当前源码事实 | 工程裁决 |
|---|---|---|
| Registry | `BTreeMap<String, Arc<dyn Handler>>`；production只有navigation registration | 无descriptor/schema/owner/capability/unregister/drain，仍是单family桥 |
| Request | V1为ABI version、operation ID、JSON payload | 无request ID、idempotency、principal、priority、公开deadline或resource claim |
| Handle | `next_handle` checked-add的裸`u64` | 不绑定session/epoch/owner/generation；foreign/stale/expired最终可退化为unknown |
| Encoded admission | ABI先按1 MiB/16,384 items/25 ms bounded decode，再调用`submit` | 避免无界DOM，但绕过`submit_json`的service permit，decode期间不占service count/bytes |
| Capacity | 默认1,024 tasks、32 prepares、4 MiB retained、每tick 8 apply、60 s TTL | private硬编码；无owner/handler/resource quota、scratch/CPU/GPU/IO预算 |
| Queued index | admission push back，claim strict front；stale惰性跳过并在capacity边界compact | 已消除随机HashMap选择；只有单级FIFO，unarmed head会阻塞后项，无priority/fairness |
| Ready index | completion按到达顺序push back，apply pop front | 已消除第二处map scan；并行prepare仍按到达偶然顺序，无conflict/ordering group |
| Snapshot | owner线程同步调用，context公开`world_mut()` | “immutable”只是注释；error/panic前可改World且service会错误报告Failed |
| Prepare | generic scheduler detached closure，task只记`prepare_in_flight` | 没有task handle/lease、cancel/deadline/progress/budget context或shutdown join |
| Cancel/expiry | 立即转Cancelled/Expired、释放accounted bytes；当前在途task不再被驱逐 | worker仍持snapshot/handler并继续计算，实际resident memory脱离4 MiB账本，terminal不等于停止 |
| Completion | 每dispatch batch一个bounded sync channel；receiver vector线性probe | channel loss可terminal并释放slot，但batch数增长、逐tick扫描、send error与wake仍未闭合 |
| Apply | owner线程每tick最多8项，直接调用handler | 无time/hitch budget、transaction、compensation；error不能区分NotApplied/Partial/Unknown |
| Result | prepare预制JSON result，apply成功后原样发布 | 无commit generation、实际affected set、receipt digest或effect disposition |
| Progress | nonterminal固定0/1，terminal固定1/1 | 字段存在但不是进度；无unknown-total、unit、phase sequence或timestamp |
| Wake | session有wake registration和frame activity，但Operation未接入 | submit、completion、ready apply都可能不唤醒reactive host |
| Frame demand | 只组合asset reload与animation | Queued/ReadyToApply/near-deadline不产生Immediate/After需求 |
| ABI | V7仅submit V1、poll V2、harvest V2 | 无cancel、request V2、subscription、catalog、list或paged result |
| Output | service 4 MiB，ABI result 1 MiB | 合法内部成功可能永久无法harvest，apply已发生后无缩小/转artifact路径 |
| Maintenance | deadline/TTL/select都扫描完整task map；当前profile counter记录rows | 可观察扫描成本不等于消除O(n)；refresh error仍被`let _`吞掉 |
| Retention | Completed/Failed占task slot到TTL；pressure只删Cancelled/Expired/Harvested | active/result/tombstone同表同capacity；current inflight fix只防止错误早删 |
| Shutdown | Session先停event mirror/watchers/modules，Operation没有close/drain | handler依赖可能先卸载；Drop receiver后worker send失败仍被忽略，无final census |
| Consumer | 仅navigation四项；Bake两项固定失败，clear/restore可工作 | 没有真实重型operation；Editor 16次spin poll与in-process capability分叉仍在 |
| Tests | core增至29 test declarations，含FIFO、channel loss、inflight retention和ignored benchmark | 多worker顺序、公平、wake、cooperative cancel、shutdown、oversize ABI、soak/model仍缺；plugin test漂移 |

## 5. 对Runtime41旧结论的纠正

| 旧结论 | 当前裁决 |
|---|---|
| Queued与ReadyToApply都从HashMap任取 | 已过时；当前为两个bounded/compacted `VecDeque`，旧P1-11/P1-12因此Partial |
| completion只有receiver扫描且channel loss不透明 | 部分修正；WorkerChannelLost、batch handles和slot释放已存在，但单queue/wake/send/shutdown仍缺 |
| cancelled preparing task可能被pressure删掉 | 当前未提交修复已防止`prepare_in_flight`被驱逐，并有focused test；worker仍继续运行且内存脱账 |
| 没有Operation性能观测 | 过宽；当前有phase index ignored benchmark及deadline/TTL/completion scan-row counters，但无release receipt与预算门 |
| 真实FFI绕过raw service reservation | 保留；bounded decoder存在但仍调用`submit`，没有把DOM前reservation交给service |
| submit无wake、frame demand忽略Operation | 保留；当前wake基础设施更完整，但Operation仍未消费 |
| navigation Bake固定失败、Editor 16次poll、plugin test漂移 | 保留；当前源码逐项仍成立 |
| 48项P1/12项P2/40 Gate | 编号与authority保持；本文只重判状态，不重复计数 |

## 6. P0所有权路由：不重复登记

| canonical blocker | 唯一owner | Runtime130责任 |
|---|---|---|
| navigation Bake生产固定失败且test期待旧成功路径 | Editor19 P0-1 + Runtime08D | 提供可取消prepare/typed commit的通用合同，不伪造Bake成功 |
| Bake panel/job/asset transaction/product host未闭合 | Editor19 P0-3 + Editor09 | 提供lossless terminal/progress/cancel stream，Editor投影job与receipt |
| scheduler task越过session/DLL unload | Runtime02 P0-1 | Operation保存session-owned task lease并导出drain census |
| destroy/quiesce缺deadline/cancel/fence | Runtime Interface01 P0-05 | close admission、cancel/drain/fence接入唯一destroy state machine |
| process-wide shutdown排序 | App01 P0-3 | Operation只提供domain fence，App拥有跨service最终排序 |

## 7. Runtime41 P1当前状态

| ID | 状态 | 当前证据与裁决 | 必须重构 |
|---|---|---|---|
| RT41-P1-01 | Open | snapshot仍接收含`world_mut()`的通用context | 拆`OperationSnapshotContext`并以compile-fail证明只读 |
| RT41-P1-02 | Open | apply仍只返回`Result<()>`，失败副作用未知 | effect class、commit point、disposition与compensation receipt |
| RT41-P1-03 | Open | result在prepare阶段预制，apply只确认成功 | commit后形成final result/receipt并记录generation |
| RT41-P1-04 | Open | registry没有descriptor/schema catalog | immutable descriptor、payload/result schema、policy与capability |
| RT41-P1-05 | Partial | task已保存registry canonical key，常量也集中；输入仍只判空 | typed namespaced ID、长度/Unicode/canonical/version规则 |
| RT41-P1-06 | Open | construction-only注册，无owner lease/unregister/drain | owner-scoped registration generation与retire协议 |
| RT41-P1-07 | Open | retry总是新handle，无stable request identity | RequestId、idempotency key、intent fingerprint与replay |
| RT41-P1-08 | Open | handle仍是session局部裸`u64` | session epoch/owner/generation-qualified opaque identity |
| RT41-P1-09 | Open | caller只需session handle和字符串ID | principal/capability/origin/audit/redaction preflight |
| RT41-P1-10 | Partial | Rust内部deadline+timer成立，ABI没有deadline/priority/SLO | Request V2 policy与明确clock domain |
| RT41-P1-11 | Partial | queued FIFO与O(1) claim已落地；managed验证、priority/replay未闭合 | sequence + per-priority FIFO + aging/fairness trace |
| RT41-P1-12 | Partial | ready FIFO替代map scan；多worker仍按completion arrival | conflict key、ordering group、generation validation与replay policy |
| RT41-P1-13 | Open | QueueDepth仍是提交时`tasks.len()+1`且含tombstone | 可信queued-ahead/active class或明确indeterminate |
| RT41-P1-14 | Open | 只有session总count/bytes/prepare上限 | global/owner/handler/principal/resource分层quota |
| RT41-P1-15 | Open | limits仍为crate-private硬编码 | validated product/platform policy与effective diagnostics |
| RT41-P1-16 | Open | `max_owner_applies_per_tick`仍同时控制snapshot和apply | 独立count/time budget并接frame budget controller |
| RT41-P1-17 | Open | snapshot callback无耗时、分配或访问预算 | slow-handler telemetry、deadline与owner budget debit |
| RT41-P1-18 | Open | apply callback仍可任意同步阻塞 | bounded publication或maintenance window/hitch gate |
| RT41-P1-19 | Partial | 有`prepare_in_flight`精确slot和batch handle census；无scheduler lease | session-owned task group、handle、join和owner cancellation |
| RT41-P1-20 | Partial | cancel阻止apply且在途task不再被驱逐；worker与snapshot仍继续 | CancelRequested/Stopping/Cancelled、token checkpoint和确认终态 |
| RT41-P1-21 | Partial | deadline会terminal并保留在途slot；不能撤回/中断running worker | scheduler retract + deadline token + noncooperative fuse |
| RT41-P1-22 | Open | prepare签名仍只有owned JSON snapshot | PrepareContext含cancel/deadline/progress/budget/diagnostic sink |
| RT41-P1-23 | Open | 无pause/resume/reprioritize | scheduler reprioritize与handler cooperative pause分层 |
| RT41-P1-24 | Partial | navigation clear/restore有单用途generation compare | 通用dependency/batch/coalesce/supersede key与generation input |
| RT41-P1-25 | Open | public table和gateway仍无cancel | typed cancel receipt贯穿ABI/App/Editor/in-process |
| RT41-P1-26 | Open | V1无法传deadline/priority/idempotency | V2 request、capability negotiation与legacy telemetry |
| RT41-P1-27 | Open | progress仍固定0/1或1/1 | indeterminate/units/completed/total/sequence/timestamp |
| RT41-P1-28 | Open | 只有主动poll | bounded coalesced progress + lossless terminal subscription |
| RT41-P1-29 | Open | submit使用`with_session`且不触发frame activity/wake | admission成功与wake request同一可证明事务 |
| RT41-P1-30 | Open | session demand仍只看asset reload/animation | Operation输出Immediate/After/Idle并并入accumulator |
| RT41-P1-31 | Partial | 有deadline/terminal时间和scan counters；无完整阶段时间 | submitted/snapshot/prepare/apply timestamps、wait reason与SLO |
| RT41-P1-32 | Partial | fixed status有typed phase/detail；handler/result仍是String | stable error domain/code/stage/retry/disposition/diagnostic ID |
| RT41-P1-33 | Open | service 4 MiB、transport 1 MiB差异仍存在 | transport-aware admission或paged/CAS artifact result |
| RT41-P1-34 | Open | Value在decode、计数、prepare和harvest重复serialize/clone | schema-bound owned bytes/types，一次建立size与ownership |
| RT41-P1-35 | Partial | ABI有bounded pre-DOM decode；service raw permit仍被绕过 | 唯一encoded admission permit并原子转换为task reservation |
| RT41-P1-36 | Open | harvest仍是单个JSON包 | inline/page/artifact三类ResultStore与逐页ack |
| RT41-P1-37 | Partial | 当前修复保护in-flight terminal task；active/result/tombstone仍共用task capacity | 拆active task、result store、tombstone容量和pressure policy |
| RT41-P1-38 | Open | TTL后Expired、pressure后Unknown | bounded generation-aware final tombstone/expiry receipt |
| RT41-P1-39 | Partial | foreign allocation失败可rollback harvest；无caller/result identity | result generation、idempotent retrieval token和ack receipt |
| RT41-P1-40 | Partial | channel有界且有scan counter；仍是per-batch receiver vector线性probe | 单service completion port、permit与budgeted drain |
| RT41-P1-41 | Open | worker仍`let _ = sender.send(...)` | send失败完成task lease、census和closing receipt |
| RT41-P1-42 | Partial | shared alarm/refresh存在；transition和callback仍吞refresh error | typed degraded fallback、health event与host maintenance owner |
| RT41-P1-43 | Partial | generation用于拒绝stale callback；仍`wrapping_add`且无receipt | nonzero epoch、exhaustion fuse、alarm sequence/receipt |
| RT41-P1-44 | Open | poisoned state/refresh/completion mutex仍直接`into_inner()` | no-panic transition、invariant validator与service fuse |
| RT41-P1-45 | Partial | checked add/sub和focused invariant guards增加；shipping仍大量`expect` | reservation/permit RAII、typed transition和failure census |
| RT41-P1-46 | Open | service没有Open/Closing/Draining/Closed | idempotent close admission、cancel all、drain deadline、final census |
| RT41-P1-47 | Open | dynamic teardown没有调用Operation，module先shutdown | Operation fence必须在handler/module/plugin teardown之前 |
| RT41-P1-48 | Partial | FIFO/channel/inflight tests增加到29项；关键并发与产品矩阵仍缺且plugin test漂移 | conformance、race/model、wake/shutdown/oversize/soak/product tests |

## 8. Runtime41 P2当前状态

| ID | 状态 | 当前证据与裁决 | 必须补齐 |
|---|---|---|---|
| RT41-P2-01 | Open | 无可枚举descriptor catalog | capability-safe catalog/filter/schema export |
| RT41-P2-02 | Open | 无payload-redacted service snapshot | phase/owner/bytes/age/health diagnostics |
| RT41-P2-03 | Open | request、worker、apply、ABI、Editor无共同trace ID | stable correlation/span chain |
| RT41-P2-04 | Open | String error直接进入外部文本 | safe message key与internal source chain分层 |
| RT41-P2-05 | Open | 无operator list/filter/generation pagination | bounded administrative projection |
| RT41-P2-06 | Partial | 当前有deadline/TTL/completion scan row counters | outcome/latency/peak/shutdown历史聚合与限基数维度 |
| RT41-P2-07 | Open | Instant、process timer、scheduler不可注入 | deterministic clock/timer/scheduler fixture |
| RT41-P2-08 | Partial | 行为测试增加，但9项source guard仍靠`include_str().contains()` | API/compile-fail/behavior test替换关键文本断言 |
| RT41-P2-09 | Open | 无property/fuzz/loom状态机证明 | transition/accounting/cancel-complete model suite |
| RT41-P2-10 | Partial | 1,024-task paired benchmark已写但ignored且managed evidence pending | release P50/P95/alloc/lock/frame/RSS规模包 |
| RT41-P2-11 | Open | 无typed handler SDK/conformance harness | descriptor builder、cancel/progress/effect模板 |
| RT41-P2-12 | Partial | Runtime41与两个实施记录描述状态机；无版本化operator文档 | canonical phase/ABI/retention/shutdown手册和receipt链接 |

## 9. 本地参考源码给出的工程边界

### 9.1 Unreal：取消提示不等于安全结束

`IAssetCompilingManager`分别提供remaining count、finish selected/all、best-effort cancel和`Shutdown()`；注释明确取消不保证停止，若要确认对象活动结束仍必须finish。`ProcessAsyncTasks`接受限制执行时间的参数，manager registration/unregistration和remaining-change反馈又把工作owner、主线程completion与Editor响应性连接起来。`AsyncWork`/QueuedThreadPool进一步区分queued retract/abandon、running completion和ensure completion。Zircon应吸收“cancel state、finish fence、shutdown safety是三个合同”，并把owner completion纳入frame budget；不能仅复制UE类层次或把阻塞finish当常规产品轮询。

### 9.2 Godot：执行器提供ID、优先级、组进度和wait

`WorkerThreadPool`为task/group提供ID、high-priority标记、description、completion查询、processed element count和wait。这证明大量工作需要可寻址group、可信处理数量和显式等待边界。它不替Operation决定principal、idempotency、World commit或跨ABI result，因此Zircon descriptor/control plane必须位于统一scheduler之上。

### 9.3 Bevy与Fyrox：scope和owner completion必须显式

Bevy `TaskPoolBuilder`显式线程配置，`scope`保证借用任务在返回前全部完成，spawned task的detach/drop语义要求caller明确生命周期。Fyrox core task pool以ID/channel回收结果，engine task handler把completion绑定plugin或scene node并在持有相应context时应用。两者支持Zircon保留prepare/apply方向，但也说明generic detached closure不足以证明session/plugin teardown。

### 9.4 Unity Graphics：长期任务必须传播进度/取消并在释放前fence

Light Baker worker连接parent process、周期报告progress、读取cancellation并在结束时cancel reporter、`Join`线程；AsyncTextureSynchronizer在释放NativeArray/RenderTexture前等待未完成GPU readback。可吸收的最低边界是进度、取消传播和resource-release fence。该Graphics镜像不包含Unity完整Editor job authority，本报告不对缺失源码作扩张推断。

### 9.5 Zircon的超越目标

目标不是复刻任一执行器，而是组合出它们单独没有提供的合同：typed descriptor/schema、principal/capability、request idempotency、session/owner/generation identity、deterministic fair scheduling、cooperative cancellation、explicit effect disposition、event-driven wake、transport-safe result delivery和可证明的shutdown census。性能优于Unreal也只能由同硬件、同operation、同输入、同结果质量的queue latency、owner frame time、CPU/RSS和shutdown receipt证明，不能用功能缺失或较小样例替代。

## 10. 目标架构与模块归属

```text
zircon_runtime::core::framework
  OperationTypeId / Descriptor / RequestPolicy / Status / Receipt DTO
                         |
zircon_runtime::core::manager
  OperationServiceHandle + capability-checked facade
                         |
zircon_runtime::core::runtime
  OperationCatalog
    -> AdmissionController
    -> DeterministicScheduler
    -> SnapshotPreflight(read-only owner context)
    -> OwnedPrepareTask(cancel/deadline/progress/budget)
    -> CommitCoordinator(conflict key + effect disposition)
    -> ResultStore(inline/page/artifact)
    -> ShutdownFence + DiagnosticsReceipt
                         |
zircon_runtime_interface / App / Editor
  versioned ABI + event stream + gateway projection; no second authority
```

Operation lifecycle、scheduler adapter、result store和shutdown fence属于`core::runtime`；稳定DTO/descriptor shape属于`core::framework`；跨模块访问只经`core::manager` facade。dynamic session负责装配和frame/wake桥，Editor只投影job/UI，navigation只实现domain handler。禁止把新控制面继续堆进当前859行`operation/service.rs`，也禁止在Editor或plugin创建第二个operation registry。

## 11. 分阶段重构计划

### OP-M0 · Truth freeze与RED资格

1. 冻结当前V1/V2 ABI、两个FIFO、harvest与inflight retention行为；把当前untracked test纳入受管验证。
2. 新增snapshot mutation compile-fail、multiworker apply order、reactive no-animation wake、destroy-with-running-prepare和4 MiB internal/1 MiB ABI result RED。
3. 保持navigation Bake固定失败与stale plugin test可见，不以mock成功或compat shim掩盖产品断路。

### OP-M1 · Descriptor、identity与权限

实现`OperationTypeId`、versioned descriptor/schema、owner registration lease、RequestId/idempotency、session epoch/generation handle、principal/capability与typed error/disposition。V1仅作为可观测legacy adapter。

### OP-M2 · Admission与确定性scheduler

统一encoded request permit；拆active/result/tombstone容量；建立global/owner/handler/resource quota、priority FIFO、aging/fairness、dependency/supersede和conflict/ordering group。每次选择输出可重放trace。

### OP-M3 · 只读snapshot与owned prepare

硬切只读snapshot context；prepare通过Runtime02 session-owned task group执行，并接cancel/deadline/progress/budget/diagnostic context。queued cancel可撤回，running cancel必须等worker确认或返回noncooperative health fault。

### OP-M4 · Typed commit与result store

apply改为infallible publication或显式transaction/compensation receipt；final result在commit后形成。小结果inline，大结果分页或CAS artifact，所有路径都在admission时证明可交付。

### OP-M5 · Wake、ABI与gateway parity

submit、completion、ready apply和deadline触发coalesced wake；Operation参与session frame demand。ABI增加Request V2、cancel、subscribe/unsubscribe、catalog query与paged/artifact retrieval；dynamic/App/Editor/in-process保持同一能力和错误合同。

### OP-M6 · Shutdown与plugin lifecycle

实现Open/Closing/Draining/Closed、owner close、cancel all、bounded drain和final census。Session destroy先取得Operation fence，再卸载handler依赖/module/plugin；超时返回未完成handle、stage和disposition，不能静默Drop。

### OP-M7 · 两个真实产品与性能资格

先把navigation Bake接入pure prepare artifact、generation commit、真实progress/cancel和Editor event-driven job；再接一个非navigation重型consumer。以race/model、overload/fairness、timer failure、ABI compatibility、24h soak和同场景benchmark关闭资格门。

## 12. Required资格门当前状态

| Gate | 状态 | 当前裁决 / Required evidence |
|---|---|---|
| RT41-G01 | Pass | 本轮47个Zircon聚焦文件、13个owner plan、13个reference已重取路径/metrics/fingerprint |
| RT41-G02 | Fail | 所有operation必须有validated descriptor、owner、schema、effect class |
| RT41-G03 | Fail | snapshot context在类型层不能获得World mutation authority |
| RT41-G04 | Fail | apply commit point与receipt disposition一致 |
| RT41-G05 | Fail | pre-commit error/panic必须证明live state不变 |
| RT41-G06 | Fail | post-commit failure不得报告普通Failed/NotApplied |
| RT41-G07 | Fail | request ID + intent fingerprint支持幂等retry/replay |
| RT41-G08 | Fail | handle绑定session epoch/generation并稳定分类foreign/stale |
| RT41-G09 | Fail | principal/capability在snapshot前验证并留下audit |
| RT41-G10 | Fail | V2表达priority、deadline、idempotency、origin |
| RT41-G11 | Partial | 单级queued FIFO成立；priority/aging/replay trace与managed evidence未闭合 |
| RT41-G12 | Fail | fairness测试证明background/interactive均不饥饿 |
| RT41-G13 | Fail | global/owner/handler/resource quota并发不超配 |
| RT41-G14 | Fail | policy非法组合fail closed并导出effective values |
| RT41-G15 | Fail | snapshot/apply同时受count与time budget约束 |
| RT41-G16 | Fail | slow handler被测量、隔离并形成diagnostic |
| RT41-G17 | Fail | every prepare持有session-owned task lease与精确census |
| RT41-G18 | Partial | queued/ready publication cancel和inflight retention成立；running cooperative stop未实现 |
| RT41-G19 | Fail | deadline传播scheduler/handler且超时工作不会无限继续 |
| RT41-G20 | Fail | TooLate cancel返回commit/disposition证据 |
| RT41-G21 | Fail | progress支持indeterminate/units/sequence，禁止伪0/1 |
| RT41-G22 | Fail | terminal event lossless、progress bounded/coalesced |
| RT41-G23 | Fail | submit/completion/deadline/ready apply触发正确wake |
| RT41-G24 | Fail | reactive host中无animation operation也能完成 |
| RT41-G25 | Fail | dynamic与in-process gateway能力/错误一致 |
| RT41-G26 | Fail | cancel/deadline/progress在旧host安全协商或拒绝 |
| RT41-G27 | Partial | fixed status phase/detail已typed；error stage/retry/disposition仍缺 |
| RT41-G28 | Fail | 最大合法内部结果始终可inline/page/artifact交付 |
| RT41-G29 | Partial | bounded decoder在DOM前限流；未占service count/byte permit |
| RT41-G30 | Fail | active task、result、tombstone使用独立容量/retention |
| RT41-G31 | Partial | foreign allocation失败可rollback；caller/result generation与幂等ack缺失 |
| RT41-G32 | Partial | completion channel有界且channel loss可terminal；仍非单authority、无wake |
| RT41-G33 | Partial | shared timer/重臂存在；timer unavailable无可观察fallback |
| RT41-G34 | Fail | poison/accounting invariant失败fuse service并输出census |
| RT41-G35 | Fail | shutdown close admission并按owner cancel/drain全部task |
| RT41-G36 | Fail | session destroy在module/plugin unload前取得Operation fence |
| RT41-G37 | Fail | navigation真实Bake成功、可取消、有进度且Editor不spin |
| RT41-G38 | Fail | 第二个重型consumer通过同一conformance合同 |
| RT41-G39 | Partial | ignored paired phase-index benchmark存在；release/managed/race/soak证据缺失 |
| RT41-G40 | Pass | 本报告finding计数、frontmatter路径、索引、coverage与scoped diff-check通过后成立 |

## 13. 禁止的临时实现

1. 不允许把`world_mut()`保留在snapshot并仅靠代码评审约束“不要写”。
2. 不允许为cancel增加一个bool却仍把running task立即标为已停止。
3. 不允许把两个FIFO改名为priority scheduler，或用单worker测试证明多worker确定性。
4. 不允许在现有V1 JSON里继续塞匿名可选字段冒充versioned request policy。
5. 不允许以提高4 MiB/1 MiB常量解决大结果交付，必须分页或artifact化。
6. 不允许让Editor继续固定次数spin poll，也不允许用sleep扩大16次预算。
7. 不允许在navigation plugin建立私有Operation service、线程池或shutdown authority。
8. 不允许用compat `poll(context, handle)`恢复漂移测试；消费者必须hard cut到唯一合同。
9. 不允许把profile counter当作复杂度修复或性能优于Unreal的证据。
10. 不允许在Session Drop中忽略worker而依赖process退出回收资源。

## 14. 完成定义

当前状态为`review_complete / implementation_pending`。首个实现切片应从OP-M0开始，不应先补新按钮或新operation ID。真正完成需要同时证明：身份与权限明确、encoded admission无窗口、调度可预测且公平、取消/deadline停止真实工作、snapshot只读、commit effect可判定、结果一定可交付、reactive host必然被唤醒、session/plugin shutdown能排空，并由至少两个真实重型consumer及同语义规模证据共同验收。

本轮没有修改Runtime、Interface、App、Editor或Plugin生产源码/测试，没有运行Cargo，也没有为当前未提交Operation改动背书。后续实施前必须重取这47个Zircon文件的指纹，并先解决任何与当前`operation`工作树的lease/ownership重叠。
