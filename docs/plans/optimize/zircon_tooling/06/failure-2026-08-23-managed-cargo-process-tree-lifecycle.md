---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: managed-cargo-process-tree-lifecycle
origin_plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/10
fixing_child_dir: docs/plans/optimize/zircon_tooling/06
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/windows_job_process.py
tests:
  - cargo job 49b3c9e1c7104c7eaaa71521c3217a9e / run 057ee2aede5c4508bc95052725a6fe16
  - cargo job 15be7906105f4080ab0994c023ab3bf7 / run fbee43a42d5d4a7b97f38f1786858685
  - python -m unittest tools.session_coordinator.tests.test_cargo_runner -v
  - python -m unittest tools.session_coordinator.tests.test_windows_job_process -v
  - focused atomic-terminal cases in tools.session_coordinator.tests.test_cargo_jobs
  - cargo job 01369bac58274fa38391dd2735378b3d / run 4e07bd8f5570470c88a44903b91be5d4
---

# Tooling 06: managed Cargo process-tree lifecycle

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md`
- 来源执行切片：M0 Hub inline-test reachability / Windows Cargo acceptance
- 修复责任计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 交接原因：受管 Cargo runner 与 Windows Job Object 负责进程树收敛和终态判定；Tooling 10 不拥有该控制面实现。

## 失败现象与复现证据

两次受管 Windows 运行均启动了同一 Hub library 验证：

```text
cargo test -p zircon_hub --lib --locked --jobs 1 --message-format short --color never
```

- Job `49b3c9e1c7104c7eaaa71521c3217a9e` / run `057ee2aede5c4508bc95052725a6fe16` 以 exit code `0` 结束，但 Coordinator 报告 `status=finish_blocked`、`errorCode=cargo_process_tree_alive`。
- Job `15be7906105f4080ab0994c023ab3bf7` / run `fbee43a42d5d4a7b97f38f1786858685` 使用 `cmd.exe /d /s /c cargo test ...` 作为监督根，仍得到同一 `cargo_process_tree_alive` 结果。
- 两份 stderr 均止于依赖下载和 Rust 编译阶段，未包含 Cargo test summary；因此 exit code `0` 不能作为 Hub 258 个内联测试已执行的证据。

## 最低共享层根因

已证明的最低边界是 managed Cargo runner 对根进程退出与 Job Object 子进程树观察之间的生命周期协议：它在仍有 `rustc` 子进程时将运行标为 finished/orphaned，随后拒绝完成。是否为原子启动、子进程句柄继承或收集时序导致，仍需由 Tooling 06 在控制面中诊断。

## 架构修复验收

- 受管 Cargo run 只有在 Job Object 确认整个进程树退出后，才能发布最终 exit code 和 passed/failed 终态。
- 根 Cargo 进程先退出但 Rust 编译子进程仍在运行时，runner 必须持续收集到真实 Cargo 结论，不能产生 exit `0` 与 `cargo_process_tree_alive` 的矛盾终态。
- 用上述 Hub library 命令重放，日志必须包含 Cargo test summary，且 Tooling 10 能消费通过或真实测试失败的终态。

## 禁止临时方案

- 不得以 `cmd /c`、等待固定时长、手工终止子进程、忽略 `cargo_process_tree_alive` 或将根 exit code `0` 直接标记通过作为替代。
- 不得弱化 Hub test-reachability guard、减少测试选择范围或跳过 Windows Cargo 验收以隐藏该失败。

## 修复结果与回传

Open state: `Tooling process-tree contract fixed / Hub product gate blocked by lockfile drift`.

Coordinator commit `da0819cd1134826c26ac2afbaefd3d1c9cfc1804` implements the lower-layer repair. Cargo roots are created suspended with atomic Windows Job Object membership, the collector keeps heartbeating while it waits for every Job process instead of applying the retired local 120-second deadline, and a retained Job terminal observation is the authority for finish/release even when the PID projection has not caught up. Read or heartbeat failures terminate and close the retained Job before terminal publication.

Focused current-source validation on 2026-08-27 passed:

- `tools.session_coordinator.tests.test_cargo_runner`: 10/10;
- `tools.session_coordinator.tests.test_windows_job_process`: 8/8 real Windows Job Object cases;
- six focused `CargoJobTests` covering atomic start/resume, real runner capture/release, stale PID projection, collector authorization, live descendant rejection, and consecutive empty observations: 6/6.

The first managed Hub replay after the repair is durable run `4e07bd8f5570470c88a44903b91be5d4` on job `01369bac58274fa38391dd2735378b3d`. Unlike the two RED runs, it completed with `error_code=null`, recorded `process_tree_exited_at=2026-08-23T00:12:52.602848+00:00`, persisted `process_tree_live_pids_json=[]`, and released the job. Cargo itself exited 101 before compilation because the root `Cargo.lock` required an update while `--locked` was enforced.

That run proves the repaired Coordinator lifecycle no longer creates the contradictory root-exit-0/`cargo_process_tree_alive` terminal state, but it does not satisfy the originating Hub acceptance: no test summary was produced. The exact `zircon_hub --lib --locked` replay remains required after the separately owned root manifest/lockfile set is stable. Until then this lifecycle remains open; no `fixed-*` return or completion notification is authorized.
