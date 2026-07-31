---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: play-snapshot-transition-main-thread-stall
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/04
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/snapshot/source.rs
  - zircon_editor/src/core/play/snapshot/store.rs
  - zircon_editor/src/core/play/process_backend/mod.rs
---

# Editor04 Play snapshot与transition主线程停顿

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-550 / PERF-MVP-553 Play snapshot and transition main-thread budget
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Play transition authority 与 generation-safe commit/rollback 由 Editor04 所有，bounded snapshot persistence 联动 Runtime04/10/11。

## 失败现象与复现证据

Play start同步执行World→DynamicScene→pretty JSON、整文件write/`sync_all`/rename与process spawn；Process backend持active mutex跨materialize/spawn。`PlaySessionController.transition_gate`又跨plugin activate/deactivate及backend start/stop/poll foreign调用，慢DLL、I/O、process或callback把完整wall time计入主线程transition临界区。

本轮只完成局部止损：inactive poll单mode read；terminal child finish前释放active lock。大payload和foreign callback owner未改变。

## 最低共享层根因

Play snapshot materialization、process spawn 与 plugin/backend foreign calls 同步跨越 UI 主链、`active` mutex 和 controller transition gate，generation admission 与 foreign work 尚未分离。

## 架构修复验收

- 以world generation发布唯一immutable play snapshot artifact；serialize/materialize/spawn走Runtime11有entry+bytes+deadline的ticket，Editor04只在安全点commit。
- controller锁内只验证mode/generation并发布Starting/Stopping token；foreign work锁外，完成后generation-safe commit/rollback；stop/cancel不能等待持UI锁。
- 1/64MiB/1GiB scene与0/10ms/10s callback记录serialize/write/fsync、main/worker wall、transition/active lock hold、cancel latency、payload owners/RSS。主线程serialization/I/O=0，foreign wall不进入锁持有。
- 保持single transition authority、plugin rollback、start/stop/crash、snapshot cleanup/content、pending decision与mode事件顺序；Cargo/F4首帧/RenderDoc和独立review通过。

## 下游共享层依赖

- Runtime11 已登记同一最低共享层 failure：[`dynamic-scene-session-bounded-async-io`](../../../zircon_runtime/runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md)。其验收明确覆盖 generation-owned immutable artifact、entry/bytes/time 预算、single-flight、cancel/deadline 与 terminal observer；Editor04 不得以私有线程或通用 `EditorJobSystem` ticket 伪造该契约。
- 当前调用链已复核：`editor_event_execution/menu_action.rs` 在 `PlaySessionController::request_play` 之前同步调用 `PlaySceneSource::from_world`；后者执行 `World -> DynamicScene -> pretty JSON`。随后 `ProcessPlayBackend::start` 持 `active` mutex 执行 `materialize` 与 `PlayChild::spawn`，controller 又在 `transition_gate` 内调用 backend/activation。
- Editor04 在 Runtime11 contract 返回前继续保留 controller 的 generation-safe commit/rollback 接入与上层回归测试；不得接受现有同步路径或静态守卫为 failure fixed。

## 禁止临时方案

- 禁止用更长mutex timeout、UI禁用或detached untracked thread掩盖同步主链。
- 禁止同时保留typed snapshot、pretty JSON与第二份consumer cache作为稳定authority。

## 修复结果与回传

Open state: `等待generation snapshot ticket、锁外foreign transition与规模产品验收；局部源码守卫/rustfmt/diff不构成fixed return`。

## 产出记录与时间

- 2026-07-22：状态`open`，由逐文件性能审查登记并链接PERF-MVP-550/553。
- 2026-07-27：状态`resolving_failure`；复现并定位菜单→`from_world`→`ProcessPlayBackend::start`→controller transition 的同步调用链。复用 Runtime11 已开放的 shared-I/O failure 作为最低层依赖；移除无效来源 workflow 元数据后 `failure import` 请求 `3a40489181dc473fa80988d24d9a55e1` 对本记录返回 `matchingDiagnostics=[]`。未创建私有 worker、未声明 Cargo 或产品验收通过。
