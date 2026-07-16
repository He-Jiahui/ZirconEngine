---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: editor-full-harness-runtime-thread-budget
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_editor/src/tests/editor_event/support.rs
  - zircon_editor/src/tests/host/manager/support.rs
tests:
  - zircon_editor test binary --test-threads=1 --nocapture
  - zircon_editor test binary tests::host::manager:: --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib tasks --locked -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib worker_pool --locked -- --test-threads=1 --nocapture
---

# Runtime 11：Editor full harness 跨 Runtime 实例线程预算倍增

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Editor14 M2 / Editor15 M1 full-lib 自然结束门根因收窄
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：Editor14 已把 Editor 自建 scheduler 收敛为共享 `JobScheduler`，但最新 full harness 仍以 Runtime 三池和 asset worker 的系统级默认预算为单位累积线程；最低线程来源、预算与生命周期 owner 属于 Runtime11，不应在 Editor 测试中用串行化、缩小线程数或跳过用例掩盖。
- 更低层依赖：后续静态所有权审计已证实 registry-owned `EditorManager` 通过 `EditorUiHost.core`
  反向持有强 `CoreHandle`，形成 Runtime 自拥有环；该问题已下沉至
  [`Runtime02 service-corehandle-retention-cycle`](../02/failure-2026-07-13-service-corehandle-retention-cycle.md)。
  本交接继续拥有 Runtime11 的 task-pool/asset-worker 双预算问题，但须等待 Runtime02 先修并重测基线。

## 失败现象与复现证据

Windows official validator job `9c0bba0554b042c2b3c5a139a8bb10a7` 完成
`zircon_runtime` / `zircon_editor` test-profile 编译后，Editor test child `35772` 增长到
5547 threads。终止前线程状态为 5541 `Wait/Unknown`、5 `Wait/UserRequest`、1
`Wait/EventPairLow`，60 秒 CPU 只增加 0.015625 秒，harness 没有 summary；日志
`.codex/tmp/editor15-m1-post-render11-fix-20260713-0304.log`。

为定位最低线程来源，直接执行同一 test binary：

```text
zircon_editor-0af59361f300b435.exe --test-threads=1 --nocapture
```

`--nocapture` 证明测试不是在入口立即死锁，而是随 host/runtime fixture 运行持续累积线程：

- 进程从 8 threads 增长到 3165、3887、4091；线程创建时间以 16 条为一组持续出现。
- 当前机器 `TaskPoolOptions::default()` 解析的 `TaskPoolThreadCounts.total_threads` 为逻辑并行度 16；
  `CoreRuntime::new()` 无条件 `TaskPools::default()`，创建 io/async-compute/compute 三池，合计仍为
  16 个 worker。
- Editor 的 `EventRuntimeHarness::with_enabled_subsystems`、
  `editor_runtime_with_config_path` 等测试夹具反复调用 `CoreRuntime::new()` 并激活 asset/editor 模块；
  asset 侧又以同一 `TaskPoolOptions` 派生 `AssetWorkerPoolOptions`，另行创建 `zircon-asset-*`
  专线程，正是 Runtime11 M0.2/M2.4 尚未闭合的双预算路径。
- 窄分区 `tests::host::manager::` 在单线程 harness 中已单独增长到 549 threads，随后自然结束为
  62 passed / 17 failed / 3035 filtered out（50.78s）。这排除了 Rust test harness 并发本身，证明
  Runtime fixture/worker 生命周期即使在 `--test-threads=1` 下也会放大线程占用。
- 诊断日志：`.codex/tmp/editor14-full-nocapture-rootcause-20260713.out.log`、
  `.codex/tmp/editor14-full-nocapture-rootcause-20260713.err.log`、
  `.codex/tmp/editor14-manager-pool-leak-20260713.out.log`、
  `.codex/tmp/editor14-manager-pool-leak-20260713.err.log`。

窄分区中的 17 个产品/fixture 断言失败分别仍归其功能计划；本交接只拥有线程预算倍增与 Runtime
资源生命周期。

## 最低共享层根因

Runtime11 已证实存在两条未统一的线程来源，但它们不再被视为 549/5547 threads 无界累积的最低根因：

1. `CoreRuntime::new()` 把 `TaskPools::default()` 硬编码为每个 Runtime 实例的新三池，没有可注入的
   Runtime-owned pool budget/owner，也没有能让多个隔离 Runtime 状态安全共享同一池组的唯一构造合同。
2. `ProjectAssetManager` 又从 `TaskPoolOptions::default()` 单独推导 `AssetWorkerPoolOptions`，随后
   `AssetWorkerPool` 用 `spawn_named_thread` 创建独立线程；配置上写着 `TaskPoolIo`，执行上却没有消费
   `TaskPools::io`，因此一个 Runtime 实例可同时占用两套线程来源。
3. Runtime service registry 的强 `CoreHandle` 自拥有环由 Runtime02 负责；在该环移除前，任何池预算
   测量都会混入“旧 Runtime 永不释放”的放大效应。

Runtime11 只在 Runtime02 回传“Runtime drop 后 weak 无法 upgrade、线程回到基线”后，继续裁决
`TaskPoolOptions` 单一预算 owner 与 asset worker 收编；不得用进程级共享池替代 Runtime02 修复。

## 架构修复验收

- Runtime 构造必须具有一个当前权威的任务资源注入合同：隔离 `CoreRuntime` 状态可以显式消费同一
  `TaskPools`/Runtime task owner；默认生产入口仍可创建进程级唯一 owner，不新增旧/new 双构造 facade。
- `AssetWorkerPool` 必须执行 Runtime11 M2.4 已定裁决：优先改投 `TaskPools::io`；若保留专线程，必须由
  同一预算 owner 分配且不会与 `TaskPools` 重复记账或重复占用。
- 先消费 Runtime02 的 service-registry drop 回归；随后 Runtime11 的资源测试证明 asset event filter、
  watcher、worker pool、pending jobs 与 task-pool Arc 可有界收尾。
- 增加资源回归：连续创建、激活并关闭至少 128 个隔离 Runtime fixture，线程峰值受单一预算约束，
  结束后回到基线；同时覆盖失败激活、项目打开失败和 panic 后收尾。
- 向上复验 `tests::host::manager::` 能自然结束且不随 fixture 数线性累积线程，最后运行
  `cargo test -p zircon_editor --lib --locked -- --test-threads=1 --nocapture` 取得自然 summary。

## 禁止临时方案

- 禁止给 full gate 增加 timeout、ignore、分区替代、进程强杀后宣称通过，或仅设置
  `RUST_TEST_THREADS=1`；本次已证明单线程 harness 仍会增长到数千线程。
- 禁止把测试 Runtime 的线程数硬编码为 1、引入 test-only 无 worker stub、全局可变 Runtime 单例或
  复用带业务状态的 `CoreRuntime` 来隐藏生命周期问题。
- 禁止保留 `TaskPools` 与 `AssetWorkerPool` 两套独立预算真源，禁止增加兼容构造器、别名、旧路径重导出
  或调用点特判。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor05 M1.1 current-source 复验排队 | stale managed full harness descendant | `open / old binary hangs and retains CPU lane` | 2026-07-16 | managed job `760da1df671d4027957aa00c6c0133e5` 的 owner Session `019f5f4a-721c-7611-88d9-181b45ae3c6f` 已 heartbeat-expired 为 stale，root supervisor PID 23164 已退出，但 coordinator 仍观察 Cargo PID 37580/53520 与旧 binary `zircon_editor-7cbf6e3f9c684171.exe` PID 41288 live。该 test child 自 06:19 起约 10 分钟 CPU 仅从 163.48s 增至 163.92s、无 natural summary，并独占 compatibility pool/下一条 CPU lane，阻断 Editor05 新测试 current-source 编译。Editor05 未终止 foreign 进程，也不以旧 binary 6-mode 结果冒充新增顺序测试通过；验收需覆盖 supervisor 消失后的 descendant 有界收尾与 lane 释放。 |
| Runtime11 M2.4 fresh current-source acceptance | process task owner / asset IO owner / upward Editor lifecycle | `focused-runtime-owner-passed / full-editor-blocked-by-foreign-plugins12-deadlock` | 2026-07-16 | implementation commit `85757db1f1d06636b03c9f950297cc81cea81e42`；managed Runtime `tasks` job `8bac013da3cd4ef1966ebba35c1d722b` 24/24、`worker_pool` job `518719266bc446faa2f5e69d5522e5e5` 18/18、Editor 128-runtime exact job `3211671acd474f3ea5f49f0c8cd9e4eb` 1/1。Editor manager job `2f45366c759e45529e3396dd41d3f26c` 自然结束 63 passed / 20 failed / 3141 filtered，Runtime lifecycle anchors 全过且线程数 54；20 个失败同属外部 shader IDE dependency cycle。full-lib job `5ac5dd81c9244e7ab0b9721b1fda41c1` 在 467 项后被 foreign uncommitted Plugins12 `EnterPlayMode` runtime-event-consumer wiring 自死锁阻断：事件执行仍持有 shell guard，`begin_runtime_event_consumers()` 再次 `shell.lock()`。72 threads、CPU 无进展；协调器记录非自然 exit `4294967295` 并已释放 job/reservation。修复必须传入已投影 capability 或在释放 shell guard 后执行 consumer transition，禁止第二把 shell mutex、`try_lock` 跳过、timeout 或 test-only bypass；回传需包含 exact stack-play 和 Editor event-runtime managed gates。 |
| Runtime11 M2.4/M3 / Editor14 M2 | Runtime 三池与 asset worker 预算/生命周期 | `open-最低线程来源已收窄并路由` | 2026-07-13 | official validator 5547 threads 无 summary；`--nocapture` full diagnostic 从 8 增至 4091 threads；`tests::host::manager::` 单线程窄分区峰值 549 threads、62 passed / 17 failed 并自然退出；`CoreRuntime::new -> TaskPools::default` 与 `ProjectAssetManager -> AssetWorkerPool` 两条线程来源均由 Runtime11 所有。 |
| Runtime02 -> Runtime11 | service-registry 强引用环下沉 | `open-等待Runtime02先修后重测预算` | 2026-07-13 | 已证实 `CoreRuntimeInner.services -> ServiceEntry.instance -> EditorManager -> EditorUiHost.core -> CoreHandle.inner -> CoreRuntimeInner`；最低生命周期根因已写入 Runtime02，本交接不再把共享任务池当作 5547-thread 根治方案。 |
| Runtime02 CoreWeak 后资源基线 | Manager/full-lib 线程峰值重测 | `coreweak-unbounded-growth-removed-runtime11-budget-still-open` | 2026-07-13 | 当前 locked Editor 程序的 128 Runtime fixture first/peak/last threads=`1/23/5`；Manager suite=`1/36/4` 并自然产生 66 passed / 17 failed；full-lib 到第 1727 项始终约 26–29 threads，未复现 4091/5547 无界增长。full-lib 随后停在 Editor14 `export_wizard_panel_session_poll_finishes_queued_prestart_cancellation`，因此本交接仍保持 open：Runtime02 生命周期环已消除，但 Runtime11 的单一 task budget/asset worker 收编与 Editor14 queued cancellation 自然 summary 仍须分别完成。 |
| Editor09 M1 当前源码完整门 | 第 1755 项 export-capture 触发点归属校正 | `full-harness-natural-summary-still-open` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 编译 3157-test binary 后推进至第 1755 项，随后 `cargo_capture_and_poll_complete_on_a_single_runtime_worker` 超过 10 分钟无日志并被终止；但同一 current binary 的该 exact 为 1/1、19.65s 自然通过，独立 Windows 5000+5000 双流负载 14.8s 通过。Editor15 不改生产代码并已 fixed 回传；缺少 full summary 仍属于本记录与 Editor14 的 full-harness 累积资源/生命周期验收。 |
| Editor07 indexed animation fixture 当前源码完整门 | 编译成功后 full-lib 仍无自然 summary | `full-harness-natural-summary-still-open / 线程峰值已显著收敛` | 2026-07-14 | official validator job `d52009897886431dbdfb98f0c2fd8e30` 在 9m45s 完成 current `zircon_editor` lib-test 编译并启动 `zircon_editor-a06442e54ccbf2ec.exe`；运行期监控约 107–128 threads、working set 约 3.2 GiB，未再复现 4091/5547 threads 无界增长，但约 35 分钟后 Cargo 101，日志只有 `error: test failed` 而没有 Rust test natural summary。证据 `.codex/tmp/editor07-indexed-project-fixture-full-20260714.log`；动画 fixture 的 15/15 与 reflection 1/1 已在聚焦分区自然通过，因此本条继续只归 Runtime11/Editor14 full-harness 累积生命周期，不回推给 Editor07。 |

## 修复结果与回传

- 状态：`open / Runtime11 单一进程任务 owner 与 asset IO-pool 收编已实现并通过 focused current-source gates；等待 Plugins12 返回 Play Mode shell-lock 自死锁修复后重跑 Editor full-lib natural summary`。
- 修复后须将本文件按 failure 生命周期迁回
  `docs/plans/zircon_editor/editor/14/fixed-2026-07-13-editor-full-harness-runtime-thread-budget.md`，
  并回传 Runtime focused 资源测试、Editor manager 分区线程峰值和 Editor full-lib 自然 summary。
