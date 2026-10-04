---
status: in_progress
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_runtime/300-runtime-engineering-gap-synthesis-review.md
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
  - docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
---

# Runtime 生命周期截止时间与停机清单

## 产品合同

- TaskGraph 第一次进入 closing 时冻结其 scope 停机集合。超时后的重试必须继续报告同一集合，成功后的重复 shutdown 必须返回相同的最终 scope 清单，即使外部 scope 和 task handle 已释放。
- 动态 Session destroy 的调用方 timeout 是 event mirror、project watcher、action、wake、session scope、module、TaskGraph 和 process log teardown 共享的总预算。任一阶段耗尽预算都返回 teardown incomplete，并保留 Session 及未完成的下层 owner 供显式重试。
- 成功只在 scope task body 已静止、module shutdown 成功且 Runtime 自有 worker 全部 join 后发布；不增加兼容分支或隐式丢弃活动 Session。

## 里程碑

### M1：稳定 TaskGraph scope 停机清单

在 `EngineTaskGraphState` 中持有 closing scope 集合，并在成功转入 stopped 时保存最终 census、释放 scope 强引用。超时报告、成功重试和重复成功 shutdown 都从同一生命周期快照生成 scope 清单。

定向回归覆盖一个阻塞 scope task：首次零预算 shutdown 报告 in-flight scope；释放并丢弃外部句柄后重试报告完成状态；再次 shutdown 保留与成功重试相同的 scope census。

### M2：贯穿动态 Session teardown 的单一预算

`destroy_session_slot_with_timeout` 从入口时刻计算绝对截止时间，并将其传给 Session teardown。Session teardown 将同一截止时间传入 event mirror、project watcher 和 process log 的 retryable shutdown API，并继续从中计算 scope、module 和 TaskGraph 阶段的剩余预算；失败路径保持 slot/session 与未 join 的下层 owner 可重试。

定向回归分别覆盖 event callback、watcher join、module cleanup、session scope、Runtime worker 与 process log worker 阻塞：每个短预算 destroy 都返回 teardown incomplete 且维持 closing；释放对应 owner 后默认预算重试成功并移除 Session。

### M3：批量验证与修正

先执行 rustfmt、`git diff --check` 和源级生命周期审查。随后由父 Session 的统一验证批次运行 TaskGraph shutdown 与 dynamic session registry 定向测试，并按最低共享原因修正失败；再执行 `zircon_runtime` 的约定 crate 级门禁。仅在这些验证通过后记录 accepted 证据。

## 状态与产出记录

### LIFE-A4 未完成的硬截止时间验收

当前 `*_until` 路径已传递同一绝对截止时间，并为 watcher、event reclaim、module stopping 和 log lease 保留重试 owner；这些改动尚不能宣称完整硬截止时间合同通过。以下最低 owner 仍需实现和验证：

- `ModuleLifecycle::cleanup`、`RuntimeModuleLifecycleObserver::runtime_module_deactivating` 与旧版 event reader callback 是同步可扩展回调。合作式 deadline hook 无法终止忽略 deadline 的旧回调。需要由既有受管理执行 owner 保存单个 pending cleanup operation，超时保留同一个 operation、Session 和代码加载 owner，重试只等待原操作，不能重复调用。具有主线程语义的 World/FFI 回调必须通过明确的 Session owner 调度合同执行，不能将任意 `&mut World` 回调直接移动到裸线程。
- `diagnostic_log/sink/worker.rs` 的候选修复已于 2026-09-07 合入共享主树：worker 发出 shutdown acknowledgement 后，owner 在绝对截止时间内等待共享线程完成探针，Windows 通过原生句柄确认 TLS 析构也已结束，超时保留 JoinHandle；process log lifecycle 锁也消耗同一预算。只在线程已结束时取走并 join，避免另一个 shutdown 调用在析构仍运行时因 slot 为空而误报安全卸载。已加入输出析构阻塞与 lifecycle 锁竞争的两项候选回归；受管执行仍待完成，非 Windows 的 TLS 边界仍见 M4。
- 公共 `shutdown_process_log(timeout)` 同样在入口计算截止时间，等待 lifecycle 锁与 worker 消耗其剩余预算；无效溢出预算返回 false。`astra_life_a4_idle_log_shutdown_deadline_includes_controller_lock` 和 `astra_life_a4_idle_log_shutdown_is_bounded_during_dynamic_release` 分别覆盖另一线程持锁、最后一个 dynamic lease 持锁冲刷阻塞输出的真实并发路径，释放后校验 owner 和最终记录；这些 Rust 回归仍待受管执行。
- `SESSION_REGISTRY` 的 static Mutex 已在类型系统中要求其 Session 是 Send；缺失的是创建、访问和析构的语义线程亲和合同，不能将 Mutex 本身当作 owner-thread dispatcher。硬边界必须从 Session 创建开始固定同一执行线程。
- 验收必须覆盖不合作 callback 的单次调用、超时后 pending operation 不重复提交、worker output 在 acknowledgement 后的析构阻塞、并发 log release 锁竞争和最终成功 join。当前合作式 callback 测试不能代替上述验收。

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 shared-deadline regression | TaskGraph 多 scope drain 的绝对 deadline、超时 census 与释放后重试 | `implemented_pending_validation` | 2026-09-11 | `shutdown_until_does_not_restart_budget_for_later_scope_drains` 覆盖首阶段中段收敛、后续阶段不重置预算及 retry join；既有 scope census/repeat 回归与 scoped rustfmt 通过。受管 Runtime Cargo、非合作式 callback、Session owner queue、TLS 硬截止时间及真实 Windows host 验收仍待完成 |
| M4 Session owner command queue | 动态 Session owner/ABI 收敛 | `in_progress` | — | 有效 `ffi::handle_event` 输入在调用方按 ≤256 KiB 校验、复制后交 owner 执行，typed diagnostics 在调用方 TLS 重建；`astra_life_a4_blocked_owner_payload_event_survives_short_destroy_retries` 覆盖两次短 destroy 重试及一次有效 payload 投递，独立源码复审通过。受管 Cargo、真实 DLL code owner、其余 borrowed ABI 回调与 Windows 产品门禁仍待完成 |

### M4：Session owner command queue 与硬截止时间

Windows 线程回收的最低 owner 为 `core/runtime/tasks/thread_completion.rs`：使用借用的
`JoinHandle` 原生 HANDLE 与零等待 `WaitForSingleObject` 确认线程实际终止，再允许现有
owner 取走并 join。TaskPool、diagnostic sink、project watcher 和 App Headless 共用此探针，超时继续持有
句柄。标准库 `is_finished` 只检查主函数结果包，不能证明 TLS 析构完成；TaskPool 的
函数体退出计数也不等于已 join。非 Windows 保留标准库完成检查，TLS 硬截止时间的
平台实现与执行验收仍开放，不把本次 Windows 修复表述为跨平台完成。
依据：[Rust JoinInner](https://doc.rust-lang.org/src/std/thread/lifecycle.rs.html) 与
[Win32 wait](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject)。

受管回归集合包含 `native_join_readiness_rejects_a_finished_body_with_pending_tls_destruction`、
`task_pool_retains_unjoined_worker_while_its_tls_destructor_is_blocked`、
`astra_life_a4_log_worker_retains_join_while_tls_destruction_is_blocked`、
`astra_life_a4_destroy_retains_project_watcher_during_tls_destruction` 与既有
`headless_host_thread_exit_deadline_includes_tls_destructors`。五项 Windows 回归的阻塞
夹具具有两秒兜底，旧实现会明确失败而非永久挂起整个测试进程；这些测试尚未执行。
Headless TLS 回归及其专用夹具均限定 Windows，因为其他平台当前仍使用标准库完成检查；
非 Windows 的 TLS 硬截止时间属于后续平台实现与验收，不得由这组 Windows 证据代替。

ProjectAssetManager 的带截止时间 watcher 关闭入口在回收期间持有 watcher 集合锁，
每次完成 join 后才移除对应 owner；并发回收不能在临时空集合上报告成功。activation、
watcher 集合和 activation state 的锁竞争直接返回未完成，不无界等待控制锁。
`astra_life_a4_concurrent_watcher_shutdown_cannot_report_an_in_flight_join_as_complete`
与 `astra_life_a4_watcher_shutdown_returns_pending_when_owner_locks_are_contended`
覆盖并发关闭和控制锁竞争，释放后再次关闭成功；执行证据仍待受管批次。

共享 `ProjectWatcherAdmission` 在尝试控制锁之前设置原子 closing 状态，发布许可被占用
也不能遗漏关闭准入。项目打开和关闭的操作票据
从入口持续到准备中、退休中的 watcher 完成析构，关闭不能因这些句柄暂时不在 manager
集合中而误报成功。发布前再次检查准入并持有短期发布许可；关闭后的新项目打开返回错误。
无界关闭入口已移除，Runtime owner adapter 也消费带截止时间的可重试结果；同步发布
回调可返回未完成，不等待自身操作票据退出。定向回归新增
`astra_life_a4_watcher_shutdown_retains_in_flight_transitions_and_closes_admission`、
`astra_life_a4_project_open_cannot_install_watchers_while_shutdown_is_joining` 与
`astra_life_a4_project_open_preparation_remains_owned_until_shutdown_retry`。
`astra_life_a4_project_open_wake_can_request_shutdown_without_waiting_for_itself` 与
`astra_life_a4_project_close_wake_can_request_shutdown_without_waiting_for_itself` 使用真实
JSON 资产的公开订阅回调，覆盖打开和关闭发布中的重入关闭，以及回调退出后的成功重试。
锁竞争夹具现在保留一个实际阻塞线程，并将 25 ms 预算的调度容差限制为 100 ms。
本轮仅完成源码、格式与独立复审修正，执行与跨平台 TLS 验收仍未完成。

按依赖顺序在隔离 candidate 实施：

1. `dynamic_api/session/registry/session_owner.rs` 与 `session_owner/tests.rs`：复用既有 TaskPool 的单个 dedicated I/O worker（显式一个物理线程，不新建三个 TaskGraph pool），单个持久任务内部调用 factory 创建状态（支持 !Send 状态），只接受 Send + owned 的 command/result；提供可查询 operation receipt、固定 owner thread 和一个持久 shutdown receipt。receipt 通过 Condvar 通知唤醒，不能 spin/yield 消耗等待预算。调用方 wait 的 deadline 只限制等待，不能取消或重复提交已经开始的 cleanup。成功必须同时证明状态析构、任务 terminal、物理 worker join。
2. `registry/session_slot.rs` / `session_store.rs`：Session 从 owner factory 创建；slot 保留 controller 与 pending teardown receipt，timeout 不 take Session/owner。Action finalizer 仍在 ABI caller 上写 output，既有 allocation action barrier 保持跨 prepare/finalize/commit。
3. `session/ffi.rs` / `operation.rs` / `status.rs`：输入 slice 在 caller 上完成校验并变成 owned DTO；output pointer 只留在 caller，owner 返回 owned result。ZrStatus diagnostics 当前是 TLS 指针，必须在 owner 上转为 owned diagnostic，再在 caller TLS 重建，不能跨线程传裸 pointer。
4. 生命周期测试分别证明 factory、所有 command、cleanup 和 Drop 的 thread id 相同；!Send 状态从不转移；阻塞 cleanup 的首次 wait 与重复 wait 都在硬 deadline 返回 Pending 且调用计数为 1；释放后 receipt 完成，DLL guard 在 owner terminal+join 前仍存活。

App headless `HeadlessController` 已采用从创建开始的 owner 线程作为产品硬边界。首次取消或销毁固定共享进程 deadline，`finish_runtime_until(Instant)` 在该预算内重复唤醒原 owner 的失败清理，并保留 `runtime_destroy_receipt()`；Server 日志及 Windows console 等待复用同一截止点。窗口化 Runtime、Editor composition 与 Play 的启动失败只格式化恢复诊断，保留创建线程上的 owner，恢复入口为 `zircon_app::retry_runtime_startup_cleanup`；错误映射不执行可能阻塞的 DLL cleanup。这些路径仍待受管运行验证。registry 内部迁移必须保留 UI/FFI 既有 affinity，不能以未经审查的嵌套 dispatcher 替代 App owner。

当前隔离 candidate 已实现 owner、`core/runtime/tasks/pool.rs::shutdown_until`、`session_owner/runtime.rs` 的 Runtime 创建/owned dispatch/teardown adapter 与 `abi_status.rs` TLS 转换。cleanup panic 发布 Poisoned；正常重试只查询，不调用 cleanup；只有显式 `recover_poisoned_until` 可提交独立恢复回调。factory/action/Drop panic 的 terminal 路径仍要求物理 worker join 后才允许释放 code owner。已加入真实 Runtime event callback 阻塞、非 Send 状态线程亲和、显式失败重试、panic 恢复与 factory panic 测试，尚未 Cargo 验证。

ABI 最终迁移尚未完成：约 30 个 borrowed closure 消费者仍使用 synchronous registry。`RuntimeSessionOwner::create` 明确要求 code-owner guard；现有 ABI 调用方通过 RuntimeSession 持有 DLL 的合同必须在新 owner slot 中保留，不能用无意义 token 代替真实库 owner 或把状态 TLS 指针跨线程传递。
