---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-04
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

- 根因：Every mutating command performed a health and repository preflight before POST. A single bounded GET timeout correctly prevented submission but provided no recovery attempt, so cheap validation admission could be lost while active Cargo or supervision made the health projection transiently slow.
- 架构修复：CoordinatorClient now retries only the read-only preflight once. Two timeouts still return command_preflight_timeout with submission not_submitted and no POST, while a recovered second preflight submits the original request id exactly once. POST timeouts retain durable request-status reconciliation and are never replayed.
- 验证：Three exact local regressions passed in 2.045s and the full current client suite passed 18/18 in 1.728s; Python compilation and diff checks passed. Managed ticket 55427efa356e4164ad38d04f0521891a, source manifest d6a13d6bee52a1f91376804b15bab5e21f8fe09c2f13185fade97db98c2cda04, copy job dbfc15ac2a084628a42826b3f5c2bd59 passed 3/3 in 2.109s with exit code 0. Its focused coverage explicitly isolates preflight recovery from adjacent post-response reconciliation changes. Handoff graph before return validated 561 artifacts with 0 errors.
- 回传：All related Editor08 sessions are archived and snapshot 1130 now has current-source drift in key_chord.rs, so the historical validation admission and Cargo run were not fabricated. Editor08 can start a new current-source session and rely on the accepted bounded preflight contract.
