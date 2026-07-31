---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: run-reserved-disconnect-orphans-unstarted-job
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_jobs.CargoJobTests.test_run_reserved_disconnect_terminalizes_unstarted_orphan_with_wrapper_audit
resolved_at: 2026-07-23
---


# Coordinator01: run-reserved disconnect orphans an unstarted job

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：WOC M0 exact managed CPU `run-reserved` start.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：reservation consumption has already created coordinator-owned
  persistent state when the transport disconnects, so neither the WOC owner nor
  a generic recovery caller can truthfully invent a Cargo run or alter the
  foreign job ledger.

## 失败现象与复现证据

WOC exact CPU reservation `53a1820f328c4076a617645e436f211f` was bound to job
`b79a13edb0b24d4c9f45cd94962957c5` after the preceding Layout15 job released.
The audited reservation payload was:

- command fingerprint: `e87de6db14216b22d73a051caf521669d9ab65c8d3f3a405a9734ddc9c2068de`;
- compatibility key: `62a8709e610b70c067de1ea9fb40271a44a84b409909a03cefa3bb2aa0d3d2f0`;
- target: `E:\cargo-targets\zircon-engine\woc-m0`;
- command: `cargo test --manifest-path examples/woc/native/Cargo.toml --workspace`.

The first managed `cargo run-reserved` client invocation disconnected before a
managed run was registered. Immediate raw status evidence was:

```text
command timed out after 1810 milliseconds
{"status":"offline","error":{"code":"offline","message":"Coordinator service is offline","details":{"transport":"connection_refused"}}}
```

After the service returned, `cargo run-status` reported
`cargo_run_not_found`. The control projection recorded the job as `orphaned`
with `started_at=null`, `finished_at=2026-07-18T11:33:59.285542+08:00`,
`released_at=null`, `exit_code=null`, `command=[]` and no live PID. Independent
Win32 process inspection found no WOC Cargo or rustc process. This is not a
Cargo start or test result.

An attempted recovery changed the row to `leased`, but `cargo finish
--exit-code 1` then rejected it:

```text
{"status":"error","error":{"code":"invalid_cargo_job_status","message":"Cargo job b79a13edb0b24d4c9f45cd94962957c5 is leased; expected ['orphaned', 'running']","details":{}}}
```

WOC did not reuse the job. `cargo release` terminalized it as `released` at
`2026-07-18T03:35:18.102947+00:00`, still with `started_at=null`,
`finished_at=null`, `exit_code=null`, `command=[]`, `pid=null` and empty live
PIDs. A fresh exact reservation `89c37aa1adac44ca81e56a19112e530d`
was then created from the audited payload with the same command and
compatibility fingerprints.

The coordinator RED regression
`CargoJobTests.test_run_reserved_disconnect_terminalizes_unstarted_orphan_with_wrapper_audit`
reproduces the reservation -> consume -> wrapper timeout sequence with no
registered run, no PID, and blank execution fields. It currently fails exactly
because `CargoJobService.terminalize_unstarted_disconnect` does not exist.

## 最低共享层根因

`run-reserved` has no single durable transition for the interval after a leased
job is persisted but before `CargoRunner` has registered the managed run. A
client transport loss can therefore leave an unstarted job as `orphaned` and
make the later finish/release APIs disagree about its terminal disposition.

## 架构修复验收

- Make `run-reserved` registration and service rollover produce one explicit,
  recoverable outcome when the client disconnects before process start.
- Preserve the distinction between an unstarted wrapper failure and a Cargo
  process exit; neither projection nor recovery may imply test execution.
- Define a terminal path for a recovered unstarted lease. It must either accept
  an explicit wrapper-failure disposition or document that `release` is the
  canonical terminal operation; `finish` currently rejects that recovered state.
- Add a production-shape test covering rollover/disconnect between leased-job
  binding and managed-run registration, with no duplicate Cargo process.

## 禁止临时方案

- Do not fabricate `started_at`, PID, command, exit code, or a Cargo test
  outcome for an unstarted wrapper failure.
- Do not let the WOC owner reuse the orphaned job or impersonate its Session to
  mutate the coordinator ledger.
- Do not introduce a generic retry, compatibility fallback, or duplicate Cargo
  process to mask the missing terminal transition.

## 修复结果与回传

- 根因：The run-reserved-disconnect-orphans-unstarted-job lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
