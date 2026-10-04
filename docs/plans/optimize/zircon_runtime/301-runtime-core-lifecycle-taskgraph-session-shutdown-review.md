---
related_code:
  - zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/core/framework
  - zircon_app/src/entry/entry_runner/headless.rs
  - zircon_app/src/entry/product_shutdown/coordinator.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/dynamic_api/session
  - zircon_app/src/entry
---

# Runtime 核心生命周期、任务图、动态会话与 Host 停机纵向审查

## 审查边界

本轮只审查 `Runtime core lifecycle -> EngineTaskGraph -> dynamic session -> App shutdown`，不展开 Asset、Scene、Graphics 算法，也不审查 tooling。目标是确认“能够创建任务/会话/停机状态”是否已经构成可卸载、可恢复、可观测的生产生命周期。

已读取：

- `zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs`
- `zircon_runtime/src/core/runtime/tasks/**`
- `zircon_runtime/src/dynamic_api/session/registry/session_store.rs`
- `zircon_runtime/src/ui/module.rs`
- `zircon_app/src/entry/entry_runner/headless.rs`
- `zircon_app/src/entry/product_shutdown/coordinator.rs`
- Runtime02/157/158/193/209、Interface08/09/15/16、App 与 MVP 相关现有报告。

当前工作区存在大量并行 tracked/untracked 修改；本报告位置只作为静态证据，实施前必须重新取得 fingerprint，并标记 `source_recheck_required`。

## 当前调用链

Runtime 的 `EngineTaskGraph::try_new` 创建三类 Runtime-owned pool 和 callback dispatcher；`create_scope` 在 graph lifecycle 为 Running 时登记 scope；`shutdown` 先关闭 scope admission，再等待 scope quiescence，随后关闭并 join 三类 worker。Dynamic session 通过进程全局 `OnceLock<Mutex<SessionRegistry>>` 登记 `RuntimeDynamicSession`，调用通过 `SessionSlot::begin_action` 获得 action guard。App 另有 `ProductShutdownCoordinator` 记录产品 phase 和 transition，但当前可读代码没有证明它拥有 EngineTaskGraph、dynamic session、GPU、window 或 plugin 的实际 shutdown 调用权。`EntryRunner::run_headless` 只 compose 后返回。

## 发现

### CORE-LIFE-P0-01：产品 shutdown 状态机与 Runtime worker/session 没有共同的 terminal receipt

- **证据等级：E2。** `EngineTaskGraph::shutdown` 位于 `engine_task_graph.rs:123-160`，只能证明其自身 scope 与三类 worker pool 的 quiescence/join；`Drop` 位于 `164-171` 只关闭 admission。`ProductShutdownCoordinator` 的 transition 位于 `product_shutdown/coordinator.rs:98-175`，只更新 phase、reason、transition 和 failure ledger，未携带 Runtime task census、session allocation census、GPU completion、window/surface lease 或 plugin unload outcome。
- **影响：** App 进入 `Stopped/Completed` 的语义可能与 Runtime 仍有 private worker、timer、dedicated worker、foreign callback 或 dynamic allocation 不一致。对 DLL unload 而言，状态机记录完成不能证明没有代码仍执行。
- **重构：** `ProductShutdownCoordinator` 不应直接扩展成万能 manager；应让每个 owner 发布 typed `ShutdownReceipt`，由产品层按固定依赖顺序聚合：stop ingress -> close session admission -> cancel operations -> task graph quiescence -> GPU completion -> plugin/native drain -> window/surface retirement -> persistent flush -> process terminal。每一步都要有 owner、generation、deadline、outcome 和 remaining census。
- **硬切：** 禁止仅调用 `EngineTaskGraph::Drop` 或仅推进 Product phase 就报告 dynamic library unload 成功。
- **验证：** scope、timer、dedicated worker、callback、session destroy、window destroy 并发；分别注入 timeout、panic、late callback 和 device loss，确认最终 receipt 与真实 census 一致。

### CORE-LIFE-P0-02：`run_headless()` compose 后立即返回，不是 headless runtime loop

- **证据等级：E2。** `zircon_app/src/entry/entry_runner/headless.rs:7-10` 仅执行 `Self::compose(EntryConfig::new(EntryProfile::Headless))?`，随后 `Ok(())`。现有 M0/MVP 文档要求 runtime 接收输入、产生帧并可确定性退出；Headless 若作为 server/runtime profile，则至少需要 session、fixed/virtual clock、task pump、operation processing、diagnostic terminal 和 shutdown。
- **影响：** profile/compose 测试通过只能证明 descriptor composition，不能证明 headless server、simulation、asset loading 或 deterministic lifecycle 存在。
- **重构：** 分离 `compose`、`run_loop`、`request_stop`、`drain` 和 `terminalize`；headless 需要显式 `RuntimeProductHost`，无窗口但有 frame/tick contract，不能用空循环或立即返回冒充服务。
- **硬切：** 不允许将 `run_headless()` 的零错误返回解释为 server ready 或 runtime capability ready。
- **验证：** clean staged headless binary 启动、创建 session、执行至少一个 deterministic tick、处理 stop、排空 worker/operation、输出 terminal receipt；异常和超时必须非零退出并带归因。

### CORE-LIFE-P1-01：EngineTaskGraph 的 worker inventory 有意排除实际 worker owner

- **证据等级：E2。** `engine_task_graph.rs:99-120` 明确 inventory 只报告 Io、AsyncCompute、Compute，并注释排除 process-default、timer、dedicated worker；`shutdown` 的 `close_and_join` 只作用于 `self.inner.worker_pools`。
- **影响：** 线程预算、停机、动态 DLL unload 和内存/CPU诊断都可能少计 worker。局部 graph 是正确的 owner boundary，但不能作为全 Runtime execution authority。
- **重构：** 每一个可执行 worker 都必须注册到 `WorkerDomainRegistry`，具有 owner/generation/thread affinity/admission/cancel/join receipt；不能继续用“尚未采用合同”作为长期豁免。
- **验证：** worker census 与 OS thread/process observation 对齐；创建/退休/超时/重启和 runtime replacement 时没有 orphan worker。

### CORE-LIFE-P1-02：TaskGraph 生命周期是 graph-owned，但 Runtime 模块组合没有证明安装它

- **证据等级：E2。** `EngineTaskGraph::try_new` 是可调用的独立构造器；已有 Runtime158/59 报告确认 TasksModule/框架 DTO 与真实 scheduler/worker owner 仍有断裂。当前扫描未找到足以证明每个 Runtime profile 都通过同一 graph 实例提交 asset、operation、render、UI、plugin 任务的统一入口。
- **影响：** 同一进程可能同时存在 graph pools、process-default pools、asset workers、render workers 和 editor workers，实际线程 budget 与 shutdown 顺序失真。
- **重构：** 在 composition receipt 中记录唯一 `ExecutionRuntimeId`，所有 subsystem worker admission 必须引用它；不满足 provider 的 subsystem 应返回 `Unavailable/NotInstalled`，不能自行创建隐式 pool。
- **验证：** profile matrix（minimal/client/editor/server）统计所有 pool、线程、scope、task 数，并验证 shutdown 后为零。

### CORE-LIFE-P1-03：Dynamic session 使用进程全局 registry，句柄生命周期与 Project/BuildSet identity 仍弱绑定

- **证据等级：E2。** `session_store.rs:13-18` 为全局 `OnceLock<Mutex<SessionRegistry>>`；`try_allocate_handle` (`36-44`) 单调递增并在溢出后永久 exhausted；插入失败 (`74-90`) 会尝试 session shutdown，失败则 `abort`。已有 Interface15/16 已确认缺少完整 BuildSet/target/schema/session admission correlation。
- **影响：** 多项目、PIE、重载 Runtime DLL 或异常恢复时，裸 numeric handle 不能独立证明 owner/generation；全局 mutex 也使跨 session admission 和 shutdown 形成争用点。
- **重构：** 使用 `SessionIdentity { project, build_set, target, generation, slot }` 与显式 retirement/reuse policy；全局 registry 仅做受控 process index，真正 ownership 归于 Runtime host/session domain。句柄耗尽应成为可诊断 terminal outcome，不把 abort 作为正常资源策略。
- **验证：** 多 session 并发创建/销毁、跨项目 stale handle、generation replacement、异常 session、接近耗尽、DLL unload/reload 与 child-process fault。

### CORE-LIFE-P1-04：Session action guard 只能保护 action 区间，不能证明异步 output 与 callback 已退休

- **证据等级：E2。** `with_session_result_committed` (`session_store.rs:133-170`) 将 action、finalize、commit/rollback 串联，但 finalize/commit 后释放 guard；现有 Interface09 与 Runtime dynamic-session 报告已指出 output registry、callback、world sync、plugin event、foreign allocation 需要更长的 generation-qualified lease。
- **影响：** 一个动作已提交并释放 guard 后，异步 frame/readback/plugin callback 仍可能携带旧 session generation 继续 publish；session destroy 若只检查 action 数量，会遗漏异步 producer。
- **重构：** 分离 `ActionLease`、`OutputLease`、`CallbackLease` 和 `ShutdownLease`，统一由 session generation barrier 管理；commit 只发布 candidate，observed/ack 后才退休旧 generation。
- **验证：** action commit 后延迟 callback、output overflow、callback panic、unsubscribe race、destroy/recreate 同 handle 和 stale generation publish。

## 参考引擎对照

- Bevy `dev/bevy/crates/bevy_app/src/app.rs:84-163` 把 `App`、runner、sub-app 和 schedule update 分成明确实体；`App::update` 运行 schedule，而不是构造后立即结束。其 Plugin `build/ready/finish/cleanup` 位于 `dev/bevy/crates/bevy_app/src/plugin.rs:57-92`，说明异步 renderer readiness 与 cleanup 应有不同阶段。
- Godot `dev/godot/core/extension/gdextension_manager.cpp` 与 `gdextension_library_loader.cpp` 是 extension load/unload、版本与 library owner 的直接参考；不能只把 metadata descriptor 视为已加载。
- Unreal/Fyrox 的模块、插件和 worker shutdown 既有报告已经提供 E3 证据；本轮不重复复制 finding，只将其约束映射到 Runtime/App 的 current call chain。

## 重构依赖

`L0 current-source freeze -> L1 unified ExecutionRuntime/WorkerDomain -> L2 SessionIdentity/leases -> L3 product lifecycle aggregation -> L4 headless/runtime loop -> L5 DLL/plugin/GPU/window fault qualification`。这条链在 MVP F0-F5 之前只做 review、测试设计与最低阻断修复；不得借此先扩展高级 domain。

## 验收矩阵

| 类别 | 必须证明 |
|---|---|
| 正常启动 | compose、provider admission、session、至少一个 tick/frame、ready receipt |
| 正常停机 | stop ingress、cancel、scope quiescence、worker join、session output retire、terminal receipt |
| 异常 | task panic、callback panic、deadline、device loss、foreign output overflow、plugin failure |
| 并发 | 多 session、多 viewport、session replacement、close 与 submit race |
| 资源 | 全 worker/thread/allocation census 与 OS observation 一致 |
| 可重复性 | headless clean staged binary 两次运行得到一致 profile/build-set/terminal evidence |
| 性能 | startup/shutdown P50/P95/P99、task queue latency、lock contention、RSS/thread peak；没有数据不得宣称优于 Unreal |

## 与既有计划的关系

本报告不新增重复 canonical P0：P0-01 对应 Runtime01/02、Interface09/15 与 App shutdown owner；P0-02 对应 MVP F0/F2/F5 与 Runtime/App product host；P1 与 Runtime02/59、Interface15/16、Runtime43/57 交叉引用。后续实现前应先修正 owner manifest 和 current-source fingerprint，而不是再增加平行 facade。

## 状态与产出记录

- 2026-09-03：完成该窄纵向单元的 review-only 细查；未修改生产代码，未运行 Cargo、GUI、GPU、DLL 或压力验证。
- 状态：`in_progress`；Worker inventory、headless loop、session identity 和 product shutdown 仍未达到工程级完成定义。
