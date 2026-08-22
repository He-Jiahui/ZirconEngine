---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: prestart-supervisor-rollover-deadlock
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_supervision_actions.py
---

# Coordinator01: pre-start supervisor identity deadlocks rollover

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：authenticated rollover post-commit reload / Runtime09 managed validation admission
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败发生在 Cargo 启动前的 lease process identity 与 service lifecycle，最低共享原因全部属于 Coordinator。

## 失败现象与复现证据

- Runtime09 Job `92547181d33143498bb908cd9030619a` 于 `2026-08-22T10:57:12Z` acquire 后保持 `leased`，`started_at=null`，root PID `54608` 已退出，也没有 `cargo.start_accepted`。
- acquire wrapper 曾启动 coordinator launcher；Windows ancestry 因而保留 `54608 -> 42456 -> 52872`。Job 持久化为 `root_process_kind=cargo` 且没有 creation identity，maintenance 把 launcher、conhost 与当前 schema-65 daemon 投影为 `process_tree_live_pids`。
- Auth r5 commit `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493` 后的唯一 rollover action `27f01c0763a24c31929f5772e02be9a7` 永久停在 `executing/waitingForCargo`；其唯一 blocker 正是上述未 started lease 和 daemon PID。
- client 正确只轮询这一 action 到 300 秒外层 deadline，没有重复 preview/confirm。durable lifecycle intent未遵守其 timeout，仍为 accepted。
- `POST /control/v1/actions/27f01c.../cancel` 返回 `action_not_cancellable`，尽管 Web UI 对 executing `service.rollover` 显示“取消延后重启”。
- 恢复 gate 在另一个 owner 的真实 ephemeral Cargo Job `3aba15c6409043ab8b132f577951e438` 运行时正确拒绝 proof-bound daemon handoff；未 kill 进程、未改 DB、未删除 target/index。
- RED 回归 5/5 按预期失败：pre-start root kind/creation identity、leased TTL grace、leased rollover preservation、rollover cancel 和 durable timeout 均缺少生产行为。

## 最低共享层根因

`cargo acquire --pid $PID` 传入的是 validation supervisor，而 `CargoJobService.acquire()` 只保存裸 PID，依赖数据库默认值把它标成 Cargo root，也没有捕获 process creation identity。pre-start release/reconcile 随后对该 PID 使用完整 descendant tree，控制面 daemon 可以被误认作 Cargo。

`arm_rollover()` 又直接信任持久化的 `process_tree_live_pids_json`，并把 `leased` job 与真实 `running/orphaned` Cargo 同等视为 stop blocker；但 leased job 尚未通过 `cargo start` 建立 Cargo process identity，本应只作为 successor 需要保留的 FIFO reservation。最后，executor 的 cancel allowlist 漏掉 rollover，使这个错误等待无法从公开控制面撤销。

## 架构修复验收

- acquire 绑定 owner PID 时必须持久化 supervisor root kind 与 process creation identity；pre-start release 只检查可证明的 Cargo descendants，不得把 coordinator、PowerShell 或其它 control descendants当 Cargo。
- leased job 在 `cargo start --supervisor` 前不阻塞 rollover；successor仍须原样保留其 job/reservation/FIFO identity，不能删除 target 或伪造 terminal result。
- `reconcile_orphans` 对年龄小于 `leased_timeout` 的 leased job不得因一次瞬时空 process observation 立即 orphan；超时且 owner identity确实消失后才可回收。
- executing rollover 在仍处于 accepted/waiting-for-cargo 时可通过公开 cancel endpoint 原子取消 intent/action；一旦进入 awaiting-restart 或 shutdown handoff则 fail closed。
- rollover timeout 必须产生 durable terminal action/intent，不得只让调用方超时而后台无限 executing。
- 回归必须覆盖 wrapper owner拥有 coordinator/control child但无 Cargo child、存在真实 Cargo child、瞬时空观测、leased FIFO保留、cancel前后状态以及重启恢复。

## 禁止临时方案

- 不得手工改写 Job `92547181...`、伪造 Runtime09 owner、直接删除 retained pool，或按 PID/路径添加一次性白名单。
- 不得 kill 正在运行的真实 Cargo/rustc，也不得靠延长 timeout 掩盖永久等待。
- 不得把所有 descendant 都忽略；started supervisor 下的真实 Cargo descendants仍须阻止 finish/release/reuse/rollover。

## 修复结果与回传

Source repair is implemented under rotated Session `coordinator01-prestart-rollover-r4-20260822`:

- acquire now records a supplied owner PID as a supervisor root and captures its readable creation identity before entering the write transaction;
- pre-start and legacy leased rows use Cargo-descendant observation rather than the raw control-process tree, while the leased timeout grace applies whether or not the row has an owner PID;
- rollover preserves leased job/FIFO state without treating it as a stop blocker, but running/orphaned jobs with proven live Cargo descendants still defer handoff;
- the public cancel path accepts only reversible stop/restart/rollover intents, and a waiting rollover now atomically fails its intent/action with `lifecycle_rollover_timeout` at the declared deadline.

GREEN evidence: the five exact RED cases pass 5/5; full `test_cargo_jobs + test_supervision_actions` passes 113/113 in 314.844s; the existing orphaned ephemeral cleanup regression passes 1/1; `py_compile` and exact `git diff --check` pass. Open state is now `source fixed / managed ticket, scoped commit, proof-bound successor recovery and failure return pending`. No product Cargo was started by this repair.
