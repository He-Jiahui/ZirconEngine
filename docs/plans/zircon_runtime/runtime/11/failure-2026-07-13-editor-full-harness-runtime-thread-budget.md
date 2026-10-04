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
  [`Runtime02 service-corehandle-retention-cycle` fixed return](../../../zircon_editor/editor/14/fixed-2026-07-14-service-corehandle-retention-cycle.md)。
  本交接继续拥有 Runtime11 的 task-pool/asset-worker 双预算问题；Runtime02 回传后的重测结果与剩余门禁记录在下方里程碑表中。

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

## 2026-09-19 successor validation intake

Successor Session `failure-roll-01a084c8-runtime11-editor-full-harness-r2`
reclaimed the archived owner scope and sealed the current Runtime/Editor
source snapshot with coordinator request
`failure-roll-01a084c8-runtime11-editor-full-harness-20260919-r2`.
Static current-source ticket `28b1e7f0e9c241019dea5129687c7a64` is queued
(`status=queued`, source-manifest hash
`c1bb9979b927b7dc31218f59ff4a156d2098497bde980312f24dce98ee618ee6`). It
checks the Runtime task owner, shared IO-pool asset worker, typed event facade,
and 128-fixture lifecycle anchors with Windows rustfmt; it is not dynamic
acceptance. Managed `zircon_runtime` focused Cargo tests, 128-fixture and
Editor full-lib natural summaries, Plugins12 shell-lock repair, external
`E:\Git\zr_vm` clean source, independent C/I/M review, fixed return, and
closeout remain pending.

### 2026-09-19 corrected static checker receipt

The first successor checker ticket `28b1e7f0e9c241019dea5129687c7a64`
failed before asserting the source contract because its negative check scanned
test-only strings in `runtime.rs` and treated the fixture's
`TaskPools::default()`/`task_pools()` references as production construction.
The corrected current-source ticket `7597f62490aa4ed289f5cddcec4bb19c`
(`failure-roll-01a084c8-runtime11-editor-full-harness-20260919-r4`) sealed
the current 11-file manifest (`81039600a8f23d638ae0ae500d0cebdf3249419501edbc22f71f8f36a09381a6`)
and scopes that negative check to the production section before `#[cfg(test)]`.
Coordinator job `7604effb2335451f8cf41b31fe155b34` / run
`7597f62490aa4ed289f5cddcec4bb19c` completed with exit code 0 and terminal
output `RUNTIME11_EDITOR_FULL_HARNESS_CURRENT_SOURCE_CONTRACT_PARSE_PASS`;
cleanup was recorded complete. This is a static source-contract receipt only,
not the required Runtime/Editor Cargo acceptance or closeout.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-runtime11-editor-full-harness-r2` inspected all ten
  Rust source/test paths in the corrected successor manifest without editing
  them; each current SHA-256 still matches the sealed manifest
  `81039600a8f23d638ae0ae500d0cebdf3249419501edbc22f71f8f36a09381a6`.
- The review re-ran `rustfmt +1.94.1 --edition 2021 --config
  skip_children=true --check` over the complete scope and scoped
  `git diff --check`; both passed with markers `RUNTIME11_RUSTFMT_PASS` and
  `RUNTIME11_DIFF_CHECK_PASS`.
- The independent source probe passed as
  `RUNTIME11_EDITOR_FULL_HARNESS_INDEPENDENT_SOURCE_REVIEW_PASS`. It verified
  the Runtime-owned task-graph construction and weak lifecycle contract,
  process-default/shared `TaskPools` ownership, production removal of an
  unowned `TaskPools::default()` path, `AssetWorkerPool` binding to the IO
  task pool without `spawn_named_thread`/independent worker counts, typed asset
  events, and the required worker backpressure/drop/payload regressions. It
  also verified the Editor fixtures and 128-runtime lifecycle/panic/failure
  release anchors.
- Independent review result: **Critical=0 / Important=0 / Moderate=0**. No
  Plugins12, Runtime02, or Editor14 source was absorbed.
- This is static/source-only evidence. Fresh managed Runtime focused Cargo,
  128-fixture measurement, Plugins12 shell-lock repair, and Editor full-lib
  natural-summary gates remain pending because external `E:\Git\zr_vm` is
  dirty. Canonical `fixed-*` return, closeout, and WeCom notification remain
  pending until those dynamic gates pass.

## 2026-09-25 rolling successor r3 current-source reconciliation

The stable fixing Session `failure-roll-01a084c8-runtime11-editor-full-harness-r3`
was registered after the archived r2 retention window and claimed this record
under request `364073bcf93d47049622afac3130d136`. The current eleven-path
source manifest was re-read before any edit. Nine source paths still match the passed
static ticket `7597f62490aa4ed289f5cddcec4bb19c`; the current hashes are:

```text
zircon_editor/src/tests/editor_event/support.rs
  24fb7736ceae18eafbb2481fa711552ffc8146c57bbe857fcc8ce1fe2214e796
zircon_editor/src/tests/host/manager/runtime_lifecycle.rs
  cd8d6bc21930cc466ab93eb152ace46ad9affc15c87ea40cbbc748d2c5bc98d4
zircon_editor/src/tests/host/manager/support.rs
  466426fba5bdf72d2f8f57e1cc3db690c4d5ca86143e1c6693fa1f1652d84228
zircon_runtime/src/asset/facade/event.rs
  83c66535b296a8f16b3f29ccf7070fe934929e8f4c42703000a8ea67ebfb3348
zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  4c1f8996e5de93844c04edfabc3e8a4eaa5620e268a3534ef7f1e425a35ef1b4
zircon_runtime/src/asset/pipeline/worker_pool.rs
  1cf11505bcad8a01d2976f6cd4eec96a45706233003c4bdd666de1478347d22d
zircon_runtime/src/asset/pipeline/worker_pool/tests.rs
  a0a2bb4214985261ca950e3a56a6ef97b87cc77d6678a9b4483d59c9fa47b1a7
zircon_runtime/src/core/runtime/runtime.rs
  41715baf6d01b4f164853fb5de785577d4af3b5d9c7cb106b525775d08ca5026
zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  65092cc9680c8a1ee758713a7b0435ee7379cb1e7218aa22cab9043843359616
```

`zircon_runtime/src/core/runtime/tasks/pools.rs` currently hashes to
`1e0fe5789c0935df1ebd6e52adc9f96d2e9aaed0c1a9810354ec6187d3124fa3`, not the
static ticket's `c7e4ad3f8415a3c584f70622c115aad2315567aebe020ebfa71077b800c81442`.
The only observed delta is a foreign `#[cfg(test)]` conservative-budget test
module; r3 neither edits nor absorbs it. The failure record itself also has a
new r3 hash after this reconciliation, so the old source-manifest ticket is
not reused as current validation. The external `E:\Git\zr_vm` dirty-worktree
admission blocker remains active. Fresh independent review of the drifted
path, managed Runtime/Editor gates, Plugins12 shell-lock repair, 128-fixture
measurement, canonical fixed return, and closeout remain pending.

### r3 independent review receipt

Reviewer Session `review-runtime11-editor-full-harness-r3` re-read the current
manifest and the `pools.rs` drift. It confirmed that the drifted hash is
exclusively an appended `#[cfg(test)]` conservative-budget test module; no
production task-pool or asset-worker contract changed. The remaining manifest
hashes match the sealed values, and a dirty `runtime.rs` worktree status does
not constitute hash drift because its current bytes match the manifest.
Independent review result: **Critical=0 / Important=0 / Moderate=0**. Static
ticket/review evidence remains non-Cargo evidence; the external dirty
`E:\Git\zr_vm` admission and all dynamic/upward gates remain pending.

## 2026-09-26 successor intake (failure-roll-01a084c8-runtime11-editor-full-harness-r4)

- The stale r3 lifecycle was cancelled through the coordinator with no active lease. Successor failure-roll-01a084c8-runtime11-editor-full-harness-r4 now owns only this failure document; its document lease was acquired against base SHA-256 1622d389ca8a123227f874e61319547ae03aa3e4e3debbdb638ed1631914c6bf. No Runtime or Editor source path is leased or edited by this successor.
- The ten current source paths from the r3 manifest were rehashed before intake and retain the recorded values: editor_event/support.rs 24fb7736ceae18eafbb2481fa711552ffc8146c57bbe857fcc8ce1fe2214e796; host/manager/runtime_lifecycle.rs cd8d6bc21930cc466ab93eb152ace46ad9affc15c87ea40cbbc748d2c5bc98d4; host/manager/support.rs 466426fba5bdf72d2f8f57e1cc3db690c4d5ca86143e1c6693fa1f1652d84228; asset/facade/event.rs 83c66535b296a8f16b3f29ccf7070fe934929e8f4c42703000a8ea67ebfb3348; project_asset_manager/construction.rs 4c1f8996e5de93844c04edfabc3e8a4eaa5620e268a3534ef7f1e425a35ef1b4; asset/pipeline/worker_pool.rs 1cf11505bcad8a01d2976f6cd4eec96a45706233003c4bdd666de1478347d22d; worker_pool/tests.rs a0a2bb4214985261ca950e3a56a6ef97b87cc77d6678a9b4483d59c9fa47b1a7; core/runtime/runtime.rs 41715baf6d01b4f164853fb5de785577d4af3b5d9c7cb106b525775d08ca5026; core/runtime/tasks/thread_assignment.rs 65092cc9680c8a1ee758713a7b0435ee7379cb1e7218aa22cab9043843359616; core/runtime/tasks/pools.rs 1e0fe5789c0935df1ebd6e52adc9f96d2e9aaed0c1a9810354ec6187d3124fa3. Foreign dirty state in editor_event/support.rs, manager support, construction.rs, runtime.rs, and the test-only pools.rs module remains unclaimed; no source change is absorbed.
- The corrected static ticket and r3 review remain source-only evidence. Fresh managed Runtime focused Cargo, 128-fixture measurement, Plugins12 shell-lock repair, Editor full-lib natural-summary, independent successor review, canonical fixed return, coordinator closeout, and WeCom notification remain pending. The failure stays open.

### r4 corrected intake independent review receipt

Reviewer `/root/review_editor03_gizmo_private` re-read corrected intake snapshot 3934 (SHA-256 `3c7cc4157dcab6642cb38c2dbdca72ceea9511609ab64f40dd21a90a8d999acb`) after the event-facade hash correction. The stale r3 cancellation and absence of a source lease were verified. All ten current source hashes remain consistent with the r4 manifest, including the foreign test-only drift in `core/runtime/tasks/pools.rs`; no foreign change is absorbed. The ticket is static-only evidence: managed Runtime/Editor validation, the 128-fixture measurement, Plugins12 shell-lock repair, Editor full-lib natural-summary, canonical fixed return, coordinator closeout, and WeCom notification remain pending. Independent review result: **Critical=0 / Important=0 / Moderate=0**. The failure stays open.
