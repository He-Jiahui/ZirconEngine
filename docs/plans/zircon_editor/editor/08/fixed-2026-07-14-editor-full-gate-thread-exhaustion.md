---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: editor-full-gate-thread-exhaustion
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/14
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/retained_host/app
  - zircon_editor/src/ui/retained_host/viewport
tests:
  - cargo test -p zircon_editor --lib --locked --jobs 1
  - cargo test -p zircon_editor --lib --locked core::jobs::tests::thread_ownership_contract -- --test-threads=1
  - cargo test -p zircon_editor --lib ui::host::editor_manager_plugins_export::export_build::wizard::tests::panel_session::export_wizard_panel_session_poll_finishes_queued_prestart_cancellation --locked -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib ui::host::export_cargo_process::tests::cargo_capture_and_poll_complete_on_a_single_runtime_worker --locked -- --exact --test-threads=1
resolved_at: 2026-07-14
---


# Editor 14：全量 lib-test 裸线程守卫失败与进程资源停滞

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor03 / Editor08 M1 统一行为门
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：裸线程守卫和全量测试 worker/process 收尾属于 Editor14 线程所有权，不能在命令或事务计划中通过超时、ignore 或白名单掩盖。

## 失败现象与复现证据

Windows 受管 job `520d85713df249afae31661a7697ad07` 完成 44m55s 编译后进入全量测试，`core::jobs::tests::thread_ownership_contract::editor_production_sources_do_not_create_bare_threads` 失败。运行继续观察到 178 个失败名，并在末段长时间无新日志、45 秒 CPU 增量不足 1 秒；测试进程工作集约 3.5 GiB，未正常输出最终 summary，只能终止 test child，Cargo 报异常退出 `0xffffffff`，协调 job 以 exit 101 归档。

该现象与 2026-07-11 已记录的全量顺序运行资源耗尽形状一致，但本次同时有明确裸线程守卫失败，必须由 Editor14 查清生产 owner 是否重新引入直接/Builder/alias spawn，以及全量测试中的后台 worker/child process 是否缺少 shutdown/join。

## 最低共享层根因

最低可证实 owner 是 Editor14 的线程所有权与测试收尾协议。尚不能把所有停滞归因于单一 spawn；应先 exact 解析 guard 命中的文件，再以测试子集二分定位未终结 worker/process/ticket。其他 178 个功能失败分别写入其功能计划，本记录只负责线程/资源停滞。

## 架构修复验收

- `thread_ownership_contract` exact 输出具体违规 owner 后转绿，生产 crate direct/Builder/import-alias 裸线程零命中且白名单为空。
- 对导出进程、retained queue、viewport resolver 和 test-created runtime 增加确定 shutdown/join 证据；失败测试也必须释放 worker/child/process-tree。
- 全量 `cargo test -p zircon_editor --lib --locked --jobs 1` 能自然结束并打印 summary，不再靠终止进程；资源峰值与结束时线程数写入本记录。

## 禁止临时方案

- 禁止给裸线程 guard 加白名单、批量串行化/ignore 测试来隐藏泄漏，或恢复旧 JoinHandle/AtomicBool 旁路。
- 禁止把全量停滞记成 Editor08 命令失败；必须从最低资源 owner 修复并回传。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor14 M2 / Editor03+08 M1 | 裸线程守卫与全量测试收尾 | `open-失败已归档` | 2026-07-12 | job `520d85713df249afae31661a7697ad07`：编译 44m55s；guard 失败；全量观察 178 个失败名后日志/CPU 停滞，约 3.5 GiB 工作集，test child 异常退出 `0xffffffff`，coordinator exit 101。日志：`D:/cargo-targets/editor08-m1-rerun4-20260712.{log,err.log}`。 |
| Editor14 M2 | Guard false-positive 与隐式 worker-pool 倍增修复 | `实现完成-向上复验受阻` | 2026-07-12 | 精确重放扫描仅命中 `src/tests/editing/transaction_engine/locking.rs` 的合法锁序测试，证明原 guard 把测试源码误当生产 owner；扫描现排除规范测试目录/文件且不设白名单。进一步查明 `EditorContextBuilder::build()` 和 24 个测试夹具经 `EditorJobSystem::default/with_limits/with_bus` 各自创建默认 `JobScheduler`，会在并行 lib-test 中倍增 Rayon pool；现生产 Builder 强制注入 `CoreHandle` scheduler，旧隐式构造 API 删除，test-only `OnceLock` 共享 scheduler。静态扫描 `EditorJobSystem::{default,with_limits,with_bus}` 与无参 Builder 均 0 命中，触及 Rust 文件 `rustfmt --check` 通过。 |
| Editor14 M2 | 定向线程合同向上复验 | `blocked-by-UI03-compile` | 2026-07-12 | `cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests::thread_ownership_contract -- --test-threads=1 --nocapture` 在测试体前被 Runtime UI03 RichTable 导出边界阻断：E0432×2 + 派生 E0282×3。受管 job `3908ff20340a4e1f8e12e9a062ec6f59`，stderr `D:/cargo-targets/editor14-thread-guard-20260712.err.log`；功能失败已追加到 `editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md`。本 failure 保持 open，待 UI03 回传后重跑定向与全量自然结束门。 |
| Editor14 M2 | Token/provenance 守卫与终态记录有界化 | `局部通过-全量自然结束门待重跑` | 2026-07-12 | `core::jobs::tests` 受管 job `a51c2a0d5b83461f82c6bbfd73b4b2cd` 自然结束：22 passed / 0 failed / 3028 filtered out（4.02s）。已覆盖 direct/alias/Builder/Scope/scoped thread 全族且零白名单；完成句柄立即 tombstone 化，terminal history 256 上限、pending pin/release、ExpiredDependency、mutex tail 与快速完成许可回收均有合同。日志 `D:/cargo-targets/editor14-focused-rerun-20260712.{log,err.log}`。 |
| Editor14 M2 | 最终重编译与全量门前置状态 | `blocked-by-foreign-editor-ui-compile` | 2026-07-12 | 最后 token 边界回归落地后受管 job `8a813aefe7294e93981b3925466f08ed` 在测试体前被其他功能 owner 阻断：`paint_componentized_extension_workspace_for_test` E0432，以及 `component_registry.rs` 将 typed `UiComponentCategory/UiComponentLayoutRole` 与 `&str` 比较的 E0308×2。stderr `D:/cargo-targets/editor14-final-focused-20260712.err.log`。这些失败不属于线程/调度功能；本 failure 仍保持 open，待对应 UI 功能回传后重跑最终 focused 与全量自然结束门。 |
| Editor14 M2 / Editor15 M1 | 当前 binary full-lib 线程膨胀复现 | `open-资源停滞再次复现` | 2026-07-13 | Windows 受管 full run job `96ee7a04b19a49048cd08a3cdac2f99e` 的 test child 从 01:26:32 运行约 74.7 分钟后达到 5547 threads；连续 3 次 `% Processor Time` 采样均为 0，所有采样线程处于 Wait，harness 无 summary。该 binary 后又有源码变化，继续等待既不能形成 current-source gate，也无执行进度，故终止 child 并以 exit 1/released 归档。本轮证明隐式 worker/test-host 倍增仍未被全量自然结束门接受，不能仅以 `core::jobs` 22/22 外推 fixed。 |
| Editor14 M2 / Editor15 M1 | Render11 修复后 official validator 全量自然结束门 | `open-最新源码再次耗尽线程` | 2026-07-13 | Windows official validator job `9c0bba0554b042c2b3c5a139a8bb10a7` 在 Render11 两项编译错误修复后成功完成 `zircon_runtime` / `zircon_editor` test-profile 编译（13m51s），随后当前 test child `35772` 运行至 5547 threads；线程状态采样为 5541 `Wait/Unknown`、5 `Wait/UserRequest`、1 `Wait/EventPairLow`，CPU 在 60 秒内仅由 1083.6875 增至 1083.703125，日志始终停在 harness `Running` 且无 summary。为避免继续占用系统资源终止 child，Cargo 以 `0xffffffff` 异常退出，validator/job exit 1 并 released；日志 `.codex/tmp/editor15-m1-post-render11-fix-20260713-0304.log`。这次复现排除了 Render11 编译阻断，Editor14 仍须定位剩余 test-host/worker 创建与 shutdown/join owner。 |
| Editor14 M2 / Runtime11 M2.4-M3 | `--nocapture` 最低线程来源收窄 | `open-已转交Runtime11` | 2026-07-13 | 同一 binary 以 `--test-threads=1 --nocapture` 运行时从 8 增至 4091 threads，创建批次以当前机器 Runtime `TaskPoolThreadCounts.total_threads=16` 为单位；Editor host/runtime 夹具反复 `CoreRuntime::new()`，而 Runtime 又同时创建 `TaskPools::default()` 与 asset 专用 `spawn_named_thread` worker。`tests::host::manager::` 单线程窄分区单独到 549 threads，最终 62 passed / 17 failed / 3035 filtered out（50.78s），证明不是 test harness 并发。最低共享 owner 已写入 `docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md`；Editor14 不以 test-only 小池或共享业务 Runtime 掩盖。 |
| Editor14 M2 / Runtime02 core service registry | `EditorManager` 反向强持有 Runtime 根因下沉 | `open-已转交Runtime02` | 2026-07-13 | 静态所有权链已闭合：`CoreRuntimeInner.services -> ServiceEntry.instance -> EditorManager -> EditorUiHost.core -> CoreHandle.inner -> CoreRuntimeInner`。该 `Arc` 环解释 manager 单线程窄分区约 34 组 × 16 worker 的 549-thread 峰值，并使旧 Runtime/task pools 无法 drop。失败已写入 `docs/plans/zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md`；Runtime11 双预算项等待此环修复后重测。 |
| Editor14 M2 / Runtime02 修复后复测 | 当前源码 full-lib 精确停在 queued export prestart cancellation | `open-线程无界累积已消除-Editor14导出任务取消仍停滞` | 2026-07-13 | Runtime02 CoreWeak 硬切后的当前 locked 3153-test 程序以 `--nocapture --test-threads=1` 运行：Manager 分区自然结束，first/peak/last threads=`1/36/4`；full-lib 推进至第 1727 个测试时进程保持 26–29 threads、约 1.0 GiB working set，不再增长到 4091/5547。随后精确停在 `export_wizard_panel_session_poll_finishes_queued_prestart_cancellation`，日志与 CPU 连续约 3 分钟不变，最终仅终止本次诊断进程。该测试在占满 Export quota 后取消第二个排队任务，再等待第一项释放/finish；当前停滞 owner 位于 Editor14 `core::jobs` 的 queued cancellation/permit 收尾，不再属于 Runtime02 service 强环。日志 `.codex/tmp/runtime02_editor_full_lib.stdout.log` 与 `.codex/tmp/runtime02_editor_full_lib.stderr.log`。 |
| Editor14 M2 / Editor09 M1 | current full-lib 第 1755 项触发点复核 | `open-full-harness仍无自然summary` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 成功编译并推进到第 1755 项后停在 export-capture 单 worker 测试；确认重复停滞后终止，job `-1/released`。同一 current binary 的该 exact 1/1、19.65s 自然通过，证明 Editor15 功能本身不是最低故障；完整进程历史状态/资源收尾仍由 Editor14 与 Runtime11 接管，禁止给该 exact 加 timeout/ignore。 |
| Editor14 M2 / Runtime02 向上复验 | queued cancellation 与 Windows 高输出夹具修复 | `fixed-两个确定性停点精确通过` | 2026-07-13 | queued-cancel 下层任务合同 1/1 通过，真正停点是测试 `BlockingRunner` 只收到一次 release、但 sender 在 `finish()` 前未断开，后续 pipeline stage 永久等待；发送后显式 drop sender，exact 1/1、0.01s。下一停点的 Windows `for ... & for ...` 把第二循环嵌套到第一循环，3×3 探针为 3+9 行，5000 配置实际约 2500 万 stderr 行；括号分组后 exact 1/1、9.40s。两项都修测试夹具语义，不恢复裸线程、不加 timeout/ignore、不改变生产调度协议。 |
| Editor14 M2 / Runtime02 向上复验 | 当前 3157-test full-lib 自然 summary | `fixed-资源停滞验收完成-功能失败独立` | 2026-07-14 | `runtime02_zircon_editor_full_after_export_fixtures.exe --nocapture --test-threads=1` 自然结束：2975 passed / 144 failed / 38 ignored，2833.59s；exit 101 来自功能断言，不再由终止进程产生。50ms 监控 first/min/peak/last threads=`1/1/1657/59`，观测工作集峰值约 1.12 GiB；旧 4091/5547 threads 持续累积与两个确定性停点均未复现。144 项按 UI/Editor 功能 owner 独立处理；峰值预算继续留在 Runtime11，不阻断本 failure 的“自然结束并有 summary”验收。日志 `.codex/tmp/runtime02_editor_full_after_export_fixtures.{stdout,stderr}.log`。 |

## 修复结果与回传

- 根因：Runtime service registry 先前由 EditorManager 反向强持有 CoreHandle，导致 CoreRuntime/task pools 无法释放；CoreWeak 修复后暴露出两个确定性测试夹具问题：queued Export 的 BlockingRunner 在一次 release 后仍保留 sender，使后续 pipeline stage 永久等待；Windows 高输出命令未分组两个 for 循环，使第二循环嵌套执行并放大为约 2500 万行 stderr。
- 架构修复：保留 Runtime02 已完成的 CoreWeak 服务注册硬切，在操作边界显式 upgrade；Editor14 queued cancellation 夹具在一次性 release 后显式断开 sender；Windows cargo capture 夹具用括号分组独立 stdout/stderr 循环。未增加 timeout、ignore、白名单、兼容 shim 或生产线程/捕获绕行。
- 验证：当前源码 queued cancellation exact 1/1（0.01s），cargo capture exact 1/1（9.40s），既有 output-capture 合同 2/2；3157-test full-lib 以 --nocapture --test-threads=1 自然结束：2975 passed、144 failed、38 ignored、2833.59s，线程 first/min/peak/last=1/1/1657/59，旧 4091/5547 持续增长和永久停滞未复现。
- 回传：Editor14 的资源增长与 harness 永久停滞 failure 已修复并回传 Editor08；144 项 UI/Editor 功能断言由各功能计划独立处理，Runtime11 的瞬时线程峰值预算继续保持独立 open。
