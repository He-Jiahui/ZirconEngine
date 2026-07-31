---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: coordinator-health-preflight-validation-admission-timeout
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/client.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision
tests:
  - focused Coordinator01 client preflight timeout/no-submit regression
  - focused health endpoint latency regression while supervised Cargo is active
  - validation-copy admission after a recovered preflight timeout
---

# Coordinator01: health preflight can prevent validation admission

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：M3 source-bound immutable Cargo validation admission
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor08 only requests a managed validation-copy/session record. The bounded
  health preflight, command admission, and supervision latency are coordinator-owned behavior;
  a plan executor must not bypass them with direct Cargo or unleased records.

## 失败现象与复现证据

At `2026-07-27 06:19:24+08`, the attempted registration of the one-file Editor08 source-static
output record returned:

```text
code: command_preflight_timeout
command: session.register
phase: preflight
submission: not_submitted
requestId: ef2d1a07f41e45fdb20998082ada5be1
```

After the supervisor had reported `tray.recovery_clear`, an independent single-file
`snapshot.create` request failed the same preflight at `2026-07-27 06:22+08` with
`requestId: 883c6f41241845858d8503e58e0c3e68` and `submission: not_submitted`.

The error is emitted by `CoordinatorClient.command` before its POST to `/command`; the requested
Session therefore did not exist and no document could legally be written. Earlier M3
`validation-copy materialize-cargo` submissions also timed out without producing a visible
validation-copy record for the union Session. At `2026-07-27 06:4x+08`, a fresh Plugin List
materialize request likewise exceeded the client timeout after 64 seconds. A subsequent
control-snapshot query found no `validationCopies` row for
`editor08-plugin-list-canonical-route-r1-20260727`; this outcome is intentionally recorded as
unconfirmed/no persisted admission, not as a Cargo failure. During the interval the supervisor exposed
`tray.recovery_circuit_open`; later `status` reported `circuitOpenUntil: null`,
`failureCount: 0`, and `lastReasonCode: tray.recovery_clear`. Natural recovery is observation,
not evidence that bounded command admission is reliable under load.

## 最低共享层根因

`tools/session_coordinator/client.py` performs repository/health verification before every
mutating command. If that GET exceeds the client deadline, it returns a correct no-submit error
but gives a plan executor no durable validation request, retry contract, or bounded health
guarantee. The coordinator health/supervision path must remain responsive while it projects
active Cargo and recovery state; otherwise source-bound validation cannot be admitted even when
the requested action itself is cheap and the CPU lane is unrelated.

## 架构修复验收

- Coordinator01 identifies and bounds work on the health/preflight path so a supervised active
  Cargo run cannot make a normal mutating admission exceed the documented deadline.
- A focused test injects a slow health response and verifies a typed no-submit result with no
  POST side effect; a recovered retry then creates exactly one Session or validation-copy record.
- A focused active-Cargo regression verifies Session registration and validation-copy admission
  remain observable and bounded, or return a structured retry-at contract without a lost request.
- Editor08 retries only after the Coordinator01 fix is reviewed and returned; successful
  materialization must still bind the pinned `zr_vm` descriptor and a fresh immutable manifest.

## 禁止临时方案

- Do not run Cargo in the shared worktree, manually create coordinator rows, or write plan
  artifacts without a granted Session lease.
- Do not blindly replay a timed-out mutation: `submission` must be checked first to avoid
  duplicate validation copies or sessions.
- Do not treat `tray.recovery_clear`, a successful status read, or an unrelated Cargo admission
  as a fixed return for this failure.

## 修复结果与回传

Open state: `待 Coordinator01 health/preflight latency and durable admission repair`.
Editor08 has only static M2/M3 evidence; it has no immutable source-copy id, managed Cargo
terminal result, review, fixed return, or commit claim from this failure.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | M3 validation admission | open | Captured `command_preflight_timeout` for `session.register` with explicit `submission: not_submitted` and request `ef2d1a07f41e45fdb20998082ada5be1`; read `CoordinatorClient.command` to prove POST is skipped after a preflight timeout. Subsequent supervisor recovery was recorded but not accepted as a repair. |
| 2026-07-27 | Failure-record snapshot | open | A second, one-file `snapshot.create` request failed at the same preflight after `tray.recovery_clear`: request `883c6f41241845858d8503e58e0c3e68`, `submission: not_submitted`. This rules out validation-copy input size as the sole trigger. |
| 2026-07-27 | Plugin List materialize admission | open | A materialize request for immutable Plugin List validation exceeded the client timeout after 64 seconds. Read-only `control snapshot` then showed no `validationCopies` entry for `editor08-plugin-list-canonical-route-r1-20260727`; no Cargo job, source-copy id, or retry claim is inferred. |
| 2026-07-27 | Plugin List idempotent reattempt | open | After the no-copy control-snapshot check, the single permitted retry returned `command_preflight_timeout` before submission: request `357fd60e48024148a5777ab832e60c03`, command `validation_copy.materialize_cargo`, phase `preflight`, `submission: not_submitted`. The current source therefore remains unvalidated but has no ambiguous queued copy. |
| 2026-07-27 | Editor08 keymap immutable admission with pinned sibling source | open | Current-source snapshot `1130` fixed `key_chord.rs`=`ff3198d1...`, `keymap.rs`=`a29a0e23...`, `keymap/tests.rs`=`e8d2f84d...`; all three overlays were live-leased and attributed. `validation-copy materialize-cargo` then received the fixed zr_vm descriptor (`commit=503fb721...`, `mountPath=zr_vm`, two includeRoots) and `cargo test -p zircon_editor --lib core::commands::keymap::tests --locked --jobs 1 --color never -- --test-threads=1`, but the coordinator client exceeded its 34.4s command deadline without returning a source-copy job ID. Submission state is therefore unknown, no direct retry or worktree Cargo was performed, and no Cargo/run result is claimed. |
