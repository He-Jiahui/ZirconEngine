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

## 2026-09-01 current-source regression and repair evidence

The current source exposed a second process-tree race in `CargoJobService._reconcile_orphans`:
after a RUNNING job had a live observation, one transient empty Windows process-tree probe
combined with an empty root-liveness probe set `process_tree_exited_at` and immediately orphaned
the job. The focused regression
`test_running_job_requires_two_consecutive_empty_process_tree_observations` reproduced this
before the repair. The same probe also had to preserve immediate cleanup for a real
creation-time mismatch (`test_pid_reuse_orphans_the_original_job_and_allows_target_reuse`).

The repair keeps the durable empty observation as a candidate when the prior persisted process
tree was live, requiring the next empty observation for RUNNING jobs; an observed root
creation-time mismatch still orphans immediately. Current-source evidence after the repair:

- focused transient-empty, dead-running, and PID-reuse regressions: 3/3 passed;
- related CargoJob lifecycle cases (atomic terminal evidence, collector authorization, live
  descendant rejection, late descendant/reuse, and consecutive-empty observation): 6/6 passed;
- `test_cargo_runner` plus `test_windows_job_process`: 24/24 passed;
- `py_compile` and scoped `git diff --check`: passed.

The complete `test_cargo_jobs` module was attempted once and hit the 300-second command ceiling;
its only reported logic-adjacent failure was an existing fixture configured with the unsupported
managed root `C:\\cargo-targets`, and three runner cases reported the same storage-root error.
This is environment/configuration debt, not evidence against the process-tree repair. The
originating managed Hub `zircon_hub --lib --locked` replay and test summary remain outstanding,
so this handoff stays `open`.

## 2026-09-19 rolling successor source reconciliation

The current-source fixing Session
`failure-roll-01a084c8-tooling06-process-tree-r1` was admitted to the Tooling06
plan at baseline epoch `611`. The failure record, Cargo runner, Windows Job
Object wrapper, and three focused test modules transferred from archived owners
under fingerprint
`51bc027e2f343742f3f0ec77bedc359de5272725cc29e117bc9497be99946914`; all six
paths are leased to the successor and no foreign edits were reverted.

Current hashes are:

- `tools/session_coordinator/cargo_runner.py`:
  `67316b3c851112f3ede9da5a9cb987105174782a1f25972215ec2abcd139ea72`
- `tools/session_coordinator/windows_job_process.py`:
  `7c062bf11b07916344589ab2272418a7f565149798517596d911a4a3163e81bc`
- `tools/session_coordinator/tests/test_cargo_runner.py`:
  `d391ab5b53e2947652af752461c0234a35f6fb5436e215554685d618480051ca`
- `tools/session_coordinator/tests/test_windows_job_process.py`:
  `cc6c0cfeac1eb9ccfe34609fc3d6498a5a0bce83f1984a6e6a4538bfb394a114`
- `tools/session_coordinator/tests/test_cargo_jobs.py`:
  `a70379706f06ac935f3e96b4ff6b6cdf6f62405d5a5f013e734a25ef12096fac`

On this exact current source, the runner and Windows Job Object suites executed
24/24 with zero failures, errors, or skips. The focused CargoJob lifecycle set
(atomic launch/terminal evidence, collector registration, stale PID projection,
consecutive empty process-tree observations, live descendants, owner-finish
race, and PID reuse) executed 10/10 with zero failures or errors. The originating
Hub replay and root-lock acceptance are still separate product/dependency gates;
these local tests do not close the failure.

## 2026-09-19 managed static successor ticket

The six-path current-source overlay was submitted as ticket
`b4e70c9b836146159b27aa72920a2a19` (request
`failure-roll-01a084c8-tooling06-process-tree-20260919-r1`) with source-manifest
hash `dd61f39551367a06bbfa1056e6de0c046748e078148594347c2c6fedc81f8d96`.
Its managed contract command checks the atomic Job Object launch/collector
protocol, terminal-release authority, heartbeat while descendants remain live,
and the named runner/CargoJob regressions; it must emit
`TOOLING06_MANAGED_CARGO_PROCESS_TREE_CURRENT_SOURCE_CONTRACT_PASS`.
The ticket is static-only (`staticParseOnly=true`, `upwardAcceptance=false`) and
is queued. The originating Hub `--locked` replay, root lockfile gate, review,
canonical fixed return, and closeout remain deferred.

The first static ticket reached managed execution as job
`ef22e0c4e80341c1b7dffc74973fa972` / run
`b4e70c9b836146159b27aa72920a2a19` but exited `1` before validating the source
contract. Its checker incorrectly required the symbolic text `CREATE_SUSPENDED`;
the current implementation uses the equivalent Win32 creation flag
`0x00000004` in its `CreateProcessW` call. This is a checker-only validation
failure, not a source or Cargo failure. The corrected retry must assert the
actual `CreateProcessW`/flag and extended Job-list attribute anchors.

Corrected retry ticket `fff2517ebd0349ada225c016b8b6ca97` (request
`failure-roll-01a084c8-tooling06-process-tree-20260919-r2`) was admitted with
source-manifest hash `8b6135f3b2cabeb75c36c70e5858933b76cfa74fb56126f1fc1a75171570ae57`.
It retains the same six-path snapshot and uses the actual Win32 flag and
attribute anchors. The retry is queued; no terminal result is claimed yet.

The corrected retry executed as managed job `f225fcd303114e058a925a8a86861865`
/ run `fff2517ebd0349ada225c016b8b6ca97`, exited `0`, and emitted
`TOOLING06_MANAGED_CARGO_PROCESS_TREE_CURRENT_SOURCE_CONTRACT_PASS`.
Coordinator cleanup completed. The first checker-only failure is retained as
diagnostic evidence; this pass validates the current source contract only and
does not replace the required managed Hub replay, root lockfile gate, review,
canonical fixed return, or closeout.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-tooling06-process-tree-r1` inspected the five current
  source/test paths in the corrected successor manifest without editing them;
  all hashes remain byte-identical to the sealed manifest
  `dd61f39551367a06bbfa1056e6de0c046748e078148594347c2c6fedc81f8d96`.
- `py_compile` passed for the complete scope, scoped `git diff --check` passed
  as `TOOLING06_DIFF_CHECK_PASS`, and the independent source probe passed as
  `TOOLING06_PROCESS_TREE_INDEPENDENT_SOURCE_REVIEW_PASS`. The probe verified
  atomic `CreateProcessW` Job Object admission (`0x00000004` plus extended
  attribute `0x0002000D`), retained Job terminal authority, collector
  heartbeat/descendant handling, PID-reuse and consecutive-empty regressions,
  and the no-retired-local-deadline contract.
- As supplementary (unmanaged) evidence, the exact current source ran the
  three coordinator unittest modules: **97 tests, 0 failures, 0 errors**.
  This does not promote the result to a managed Cargo/product acceptance gate.
- Independent review result: **Critical=0 / Important=0 / Moderate=0**. No
  foreign coordinator source was absorbed.
- The originating managed `zircon_hub --lib --locked` replay and root lockfile
  acceptance remain pending (the last managed replay stopped at the locked
  lockfile gate), as do canonical `fixed-*` return, closeout, and WeCom
  notification. External `E:\Git\zr_vm` remains dirty.

## 2026-09-26 rolling successor source handoff

- Successor Session `failure-roll-01a084c8-tooling06-process-tree-r2` owns only
  this failure document under ownership-transfer fingerprint
  `476036b0ba3eb4d14e4fcf22dd45c8d903beb581874e270befe687faa6898cc6`.
  The pre-review document boundary is snapshot `3904` with manifest hash
  `0eabfa319fbad39e918d9d127a34492e91f24ac6ce882f32879b579d2f782023`.
  No foreign coordinator source edits were claimed, reverted, or absorbed.
- The five current production/test paths remain byte-identical to the corrected
  static ticket's source evidence: `cargo_runner.py`
  (`67316b3c851112f3ede9da5a9cb987105174782a1f25972215ec2abcd139ea72`),
  `windows_job_process.py`
  (`7c062bf11b07916344589ab2272418a7f565149798517596d911a4a3163e81bc`),
  `test_cargo_runner.py`
  (`d391ab5b53e2947652af752461c0234a35f6fb5436e215554685d618480051ca`),
  `test_windows_job_process.py`
  (`cc6c0cfeac1eb9ccfe34609fc3d6498a5a0bce83f1984a6e6a4538bfb394a114`),
  and `test_cargo_jobs.py`
  (`a70379706f06ac935f3e96b4ff6b6cdf6f62405d5a5f013e734a25ef12096fac`).
  Including the current document, the exact six-path manifest is
  `853318d578beb6c3fbb8406318fe49183554d2d995959350024356a244b3a811`.
- The corrected managed static ticket
  `fff2517ebd0349ada225c016b8b6ca97` remains the sole reusable current-source
  contract result and emitted
  `TOOLING06_MANAGED_CARGO_PROCESS_TREE_CURRENT_SOURCE_CONTRACT_PASS` with
  exit code `0`. Checker-only ticket `b4e70c9b836146159b27aa72920a2a19`
  remains retained as diagnostic evidence and is not reused as a source
  failure. The recorded unmanaged 97-test unittest run and independent source
  probe likewise remain static evidence, not a managed Cargo/product gate.
- The independent review receipt for the five-path manifest is
  `Critical=0 / Important=0 / Moderate=0`; it found no foreign edit to absorb.
  A fresh managed Windows `zircon_hub --lib --locked` replay with a Cargo test
  summary, root lockfile/product acceptance, canonical `fixed-*` return,
  closeout, and WeCom notification remain pending. External
  `E:\Git\zr_vm` is still dirty. Failure status therefore remains `open`.
- Final r2 review receipt: reviewer `review_editor03_gizmo_private` verified
  post-review snapshot `3905` and document SHA
  `e7a75ab4802972a14a899941f25b5571a427284c54908a0871c341b6f90acf37`,
  rechecked the six-path manifest and all static-ticket boundaries, and found
  **Critical=0 / Important=0 / Moderate=0**. This review does not promote the
  deferred managed Hub/product or closeout gates.
