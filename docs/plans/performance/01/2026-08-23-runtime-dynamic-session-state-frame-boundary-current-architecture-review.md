---
related_code:
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-runtime-host-intent-outbox-transaction-architecture-review.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-session-ui-surface-routing-publication-current-architecture-review.md
  - docs/plans/zircon_runtime/runtime/03-ecs-and-world-execution.md
  - docs/plans/zircon_runtime/runtime/07-diagnostics-and-profiling.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-task-graph-and-job-system.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameEngine.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
tests:
  - current state owner 1 of 1 Rust file and 4 inline tests reviewed
  - FFI session lock, session store, core shutdown and time-diagnostic call chains reviewed
  - focused source fingerprint and scoped diff ownership check passed
  - current-source Cargo, WPR, allocator, power and F4 RenderDoc pending
doc_type: implementation-evidence
status: source_reviewed_structural_plan_dynamic_blocked
---

# Runtime dynamic session state与frame boundary复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/state.rs`当前**1/1**个Rust文件、**706行、27,229 B、4 tests**，
SHA256为`ef224a8da91429fb482b9b68c8b57211aeedc2cd62a3ff2ea1314e0c0643718a`，基准HEAD为
`1354e50da53db3dad1dc25a6c9e375942ba04d35`。同时沿`ffi -> session_store -> state -> CoreRuntime/LevelSystem/
RuntimeRenderBridge/UiSurfaceSet`复核锁范围、帧推进、输出事务和销毁。文件中host-request borrowed page、计数器和
runtime UI IME聚合属于本Session前序M0/M1工作，本报告保留并引用其独立证据，不把已有diff冒充本轮新增优化。

## 当前源码判定

### P0：单一session临界区覆盖完整simulation、render与宽query

`with_session_activity_result_finalized`在`SessionSlot` mutex内执行action；因此`tick_frame`持锁串行完成time、asset
reload、`LevelSystem::tick`、operation apply、input begin-frame及定期diagnostic snapshot。`capture_frame`和
`present_viewport`又在同一锁内完成scene/UI extract及render bridge submit/present；accessibility query在锁内
rebuild、globalize并构造宽snapshot。这个锁既是lifecycle guard又被当成frame/world/render/UI/output authority，
使慢system、shader/pipeline准备、GPU/driver等待、插件callback或大query都能阻塞同session的input、drain、destroy
和editor调用。

不能用“把`Mutex`换成`RwLock`”修复：这些路径共享`LevelSystem`、render bridge、runtime UI、transaction output和
teardown状态，读写边界并未由generation artifact定义。Runtime10应把slot锁缩为`generation + lifecycle + ticket`
提交；Runtime03/09/Render01应分别发布不可变world/UI/render generation；Runtime11只调度声明了affinity、依赖、
deadline和取消语义的工作。目标是session锁内不执行world system、foreign callback、JSON、宽snapshot或GPU submit。

### P0：frame owner是顺序调用清单，不是可编译的依赖图

当前`tick_frame`固定串行顺序为time -> asset reload -> level -> operation -> input -> diagnostics。资产reload内部虽可借
scheduler，但顶层没有frame phase packet、读写集、main/render affinity或deadline；`RuntimeOperationService`还在
world mutable closure中同步apply。多核可用性不能从“存在scheduler”推导，当前边界没有足够合同安全并行world、
plugin和extract，也无法计量每个phase的ready/queue/run/commit时间。

Runtime03/11应把一次frame编译为稳定generation schedule：输入封存、fixed/update、deferred commit、derived/world
publication、extract、render submission和诊断封存分别声明依赖与affinity；同一world mutation只有一次commit，
render/UI消费不可变generation。低负载保留串行快路，并行阈值必须由任务开销和WPR数据决定，不能按系统数量猜测。

### P0：frame demand没有覆盖所有pending owner

`frame_demand()`只合并asset reload pending与animation continuous。host-request continuation已在前序报告证明可能滞留；
operation、plugin event、UI timer/IME、render pipeline completion或async world commit也没有在这里形成统一pending fact。
把它们全部映射为`Immediate`会制造无意义tick/redraw/present和功耗回归。Runtime10应发布typed wake reason与
`Idle/After/Immediate` demand，host-only、query-only和render damage分别消费；App/Editor只为真正render damage请求
present。

### P0：诊断与输入解析仍放大主线程固定成本

`tick_time`每帧先写time diagnostics；计划触发时`tick_frame`再收集完整runtime diagnostic current store并写日志。
前序PERF-MVP-324已量化render snapshot约541 series，故这不是可忽略的日志细节。`resolve_input_manager`也通过service
handle在frame结尾及每个提交事件重复解析；前序PERF-MVP-334已将其归入generation-bound input view。Runtime07应让
always-on health counter与可选detail分层、每generation封存一次Arc，关闭capture时不做wide snapshot；Runtime12
应让session持generation-checked input endpoint，失效时一次重绑而非每event registry resolve。

### P0：零时限销毁把未完成工作转成宿主泄漏风险

`DYNAMIC_SESSION_DESTROY_DRAIN_TIMEOUT`固定为`Duration::ZERO`。任一module无法即时drain便返回false；App侧为避免wake
callback UAF可能保留registration，前序PERF-MVP-574已把失败销毁的registry/proxy无界驻留列为P0。增加任意sleep
会把退出卡顿搬到caller，并不正确。Runtime02/10/11应先detach wake、停止admission，以带deadline的quiesce ticket
等待共享TaskGraph，再原子retire generation；超时必须有typed terminal/quarantine count、bytes、age上限。

## Unreal源码依据

`LaunchEngineLoop.cpp:5712`起的`FEngineLoop::Tick`用独立named/cycle scopes封装time、message pump、performance、world
与render阶段，并以`ENQUEUE_RENDER_COMMAND`把BeginFrame和scene start提交给render thread。可转移原则是明确phase、
线程亲和性和可观测queue，而不是把整个engine tick任意并行化。

`GameEngine.cpp:1768`起的`UGameEngine::Tick`仍在game thread推进world，但在检查sky/reflection capture queue后才调用昂贵
的`AreAlwaysLoadedLevelsLoaded()`与capture update。这支持“pending fact先门控昂贵工作”；Zircon需要把asset/plugin/
UI/render completion统一为typed demand，而不是每tick扫描或一律continuous。

`SlateApplication.cpp:1670`起把tick明确拆成`PlatformAndInput`、`Time`和`Widgets`；除time-only外强制game-thread，且每段
有独立profiling scope。可转移原则是先冻结affinity和阶段合同，再决定并行；不以UE体量、线程数或常量作为Zircon
预算。

## 优化里程碑与量化验收

1. M0 instrumentation：为session lock wait/hold、frame各phase、world mutation/commit、extract、render submit、
   diagnostics、input resolve、pending reason和shutdown记录静态dense counter；profiling关闭时payload/lock/alloc为0。
2. M1 generation boundary：发布`WorldCommitGeneration -> Ui/SceneExtractGeneration -> RenderPacketGeneration`，slot锁只
   交换handle/ticket；稳定frame world/UI deep clone=0，同generation extract/render packet build不超过1。
3. M2 shared scheduling：frame phase使用Runtime11唯一bounded worker set，main/render affinity显式；不同session与
   independent system可并行，低负载serial fallback，队列count/bytes/age/deadline/cancel有界。
4. M3 demand/teardown：typed wake reason不强迫present；destroy完成`stop admission -> detach wake -> quiesce -> retire`，
   caller无无界wait，failed-session quarantine为0或硬有界。

动态矩阵为systems 0/1/16/256、entities 1/1k/100k、sessions 1/2/8、UI nodes 0/1k/100k、diagnostic off/on/capture、
stable/1% dirty/asset reload/plugin stall 0/1/16ms/10s。记录lock wait/hold、phase wall/queue/run/commit、worker occupancy、
context switches、clone/alloc bytes、wake/tick/present、p50/p95/p99、RSS和energy。验收要求stable generation rebuild=0、
session锁不覆盖world/foreign/JSON/wide snapshot/GPU submit、pending work不丢且idle无额外tick/present、shutdown在deadline
内typed结束。current-source Cargo与产品可执行文件不可得前，WPR/allocator/power和F4 RenderDoc保持pending；RenderDoc
只验GPU pass/draw/upload/像素，不替代CPU锁与调度结论。

本轮没有对`state.rs`再做无依据局部修改，也不将4条未执行Rust tests或源码复读计为动态性能通过；不迁入
`review.md`、不提交milestone、不发送完成企微。
