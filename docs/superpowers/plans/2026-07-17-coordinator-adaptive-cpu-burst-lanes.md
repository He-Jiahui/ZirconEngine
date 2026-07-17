# Coordinator Adaptive CPU Burst Lanes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce managed Cargo validation wait without weakening Session admission, shared warm-cache correctness, source-manifest verification, or disk recovery guarantees.

**Architecture:** Keep the current CPU lane as the sole warm, reusable pool and its FIFO order as the default. Add one opt-in, ephemeral CPU burst lane for a waiting low-interference `cargo check` only when a Windows resource probe confirms sustained headroom and an approved non-warm drive has at least 100 GiB free. A burst job receives an isolated target below `E:\cargo-targets\zircon-engine\burst`, is independently process-supervised, and is deleted on release. The browser projects warm/burst WIP separately, including the reason a burst slot is unavailable, while Session admission stays open.

**Tech Stack:** Python 3.14, SQLite migrations, Windows `GlobalMemoryStatusEx`/`GetSystemTimes` resource sampling, PowerShell coordinator client, React 19, TypeScript, Material UI, Node test runner.

---

## Evidence and non-goals

- 2026-07-17 live snapshot showed one running warm CPU job and five pending CPU reservations, all unbound at reservation time.
- The warm target `D:\cargo-targets\zircon-engine\pool\841a...` is reused by the active job. Recent warmed commands completed in roughly 3–22 minutes; it must not be deleted, copied, or concurrently written.
- `E:\cargo-targets` had 131 GiB free at the observation point; `D:` had only 44 GiB free, so `D:` is not a burst target candidate.
- Eight CPU samples ranged from 40.9% to 77.2% with 19.26 GiB free memory on a 16-logical-processor machine. A single sample never admits burst work; admission requires the probe’s sustained window.
- This plan does not run two Cargo processes in the same target directory, run GPU work in the CPU burst lane, convert a `cargo test` to a burst job, close Session admission, drain the daemon, or kill a managed process.

## File map

- `tools/session_coordinator/resource_budget.py`: create the bounded Windows resource probe and pure burst-eligibility decision.
- `tools/session_coordinator/cpu_burst.py`: create the isolated-target selection and warm-versus-burst admission policy; it receives a resource probe and has no database or process side effects.
- `tools/session_coordinator/cargo_jobs.py`: keep the existing durable job orchestration boundary; delegate burst-policy selection to `cpu_burst.py`, then bind, supervise, and clean the selected job without changing warm FIFO semantics.
- `tools/session_coordinator/cargo_reservations.py`: retain executable-owner and terminal-reconciliation semantics for the new execution mode.
- `tools/session_coordinator/migrations.py`: add durable reservation execution mode and one-burst uniqueness indexes.
- `tools/session_coordinator/cli.py`: add an explicit `--burst-eligible` declaration to `cargo reserve-cpu`; default callers remain warm-only.
- `tools/session_coordinator/control_plane/snapshot.py`: project bounded warm/burst lane counts and burst availability reason without commands or paths.
- `tools/session_coordinator/web/src/api/contracts.ts` and `tools/session_coordinator/web/src/api/validation.ts`: validate exact burst projection fields.
- `tools/session_coordinator/web/src/pages/OverviewPage.tsx` and `tools/session_coordinator/web/src/pages/ValidationPage.tsx`: present warm WIP, burst WIP, queue ownership, and a plain-language reason when burst is unavailable.
- `tools/session_coordinator/tests/test_resource_budget.py`: create pure probe and eligibility tests.
- `tools/session_coordinator/tests/test_cargo_reservations.py`, `tools/session_coordinator/tests/test_cargo_jobs.py`, `tools/session_coordinator/tests/test_control_snapshot.py`: cover reservation, target isolation, cleanup, and bounded projection behavior.
- `tools/session_coordinator/web/src/__tests__/contracts.test.ts` and `tools/session_coordinator/web/src/__tests__/components.test.tsx`: cover exact runtime contracts and nonblocking visual wording.
- `docs/tools/session_coordinator/validation-queue.md`: document warm versus burst policy, invariants, and operator interpretation.

## M1 — Durable resource policy and isolated target choice

### Implementation slices

- [x] Create `resource_budget.py` with these immutable public values and types:

  ```python
  BURST_MIN_FREE_BYTES = 100 * 1024**3
  BURST_MAX_CPU_PERCENT = 80.0
  BURST_MIN_FREE_MEMORY_BYTES = 12 * 1024**3
  BURST_SAMPLE_COUNT = 3

  @dataclass(frozen=True)
  class ResourceSample:
      cpu_percent: float
      free_memory_bytes: int

  @dataclass(frozen=True)
  class BurstDecision:
      allowed: bool
      reason: str
  ```

  `WindowsResourceProbe.sample()` must calculate CPU use from two `GetSystemTimes` snapshots, read available physical memory through `GlobalMemoryStatusEx`, and clamp invalid counter deltas to `100.0`. `burst_decision(samples, free_bytes, burst_active)` returns exactly one of `allowed`, `burst_active`, `cpu_headroom`, `memory_headroom`, or `disk_headroom`; it allows only three valid samples whose CPU percentage is at most 80, 12 GiB available memory, at least 100 GiB free space, and no active burst reservation.

- [x] Create `cpu_burst.py` with a pure `CpuBurstRequest` input and `CpuBurstSelection` result. `select_cpu_burst(request, decision)` returns `CpuBurstSelection(mode="warm", target_dir=None, reason=decision.reason)` unless the request is CPU, burst-eligible, `cargo check`, target-free, and `decision.allowed`. Its only burst result is `CpuBurstSelection(mode="burst", target_dir=Path("E:/cargo-targets/zircon-engine/burst") / request.reservation_id, reason="allowed")`. Keep all SQLite queries, process calls, directory creation, and cleanup out of this module.

- [x] Add migration 48. Add `execution_mode TEXT NOT NULL DEFAULT 'warm' CHECK (execution_mode IN ('warm', 'burst'))` and `burst_eligible INTEGER NOT NULL DEFAULT 0 CHECK (burst_eligible IN (0, 1))` to `cargo_lane_reservations`. Replace the active-lane index with these exact indexes:

  ```sql
  CREATE UNIQUE INDEX cargo_lane_reservations_one_active_warm
      ON cargo_lane_reservations(lane_scope, execution_mode)
      WHERE lane_scope IN ('cpu', 'gpu') AND execution_mode='warm'
        AND status IN ('leased', 'running', 'finished');
  CREATE UNIQUE INDEX cargo_lane_reservations_one_active_burst
      ON cargo_lane_reservations(lane_scope, execution_mode)
      WHERE lane_scope='cpu' AND execution_mode='burst'
        AND status IN ('leased', 'running', 'finished');
  CREATE INDEX cargo_lane_reservations_cpu_warm_fifo
      ON cargo_lane_reservations(lane_scope, execution_mode, status, created_at, reservation_id)
      WHERE lane_scope='cpu' AND execution_mode='warm';
  ```

  Leave the existing GPU uniqueness rule unchanged. Existing rows migrate as warm and must preserve their reservation IDs, commands, source manifests, target identities, and current statuses.

- [x] Extend `cargo reserve-cpu` with `--burst-eligible`; send `burst_eligible` in `cargo.reserve_cpu`. The default must be false. Reject `--burst-eligible` unless the exact reserved command begins `cargo check`, has no `--target-dir`, and the compatibility JSON has a current source manifest when a source manifest is required by the existing policy.

- [x] In `CargoJobService._reserve_lane`, retain the current warm FIFO insertion. Do not pick a burst target during reserve. Store `burst_eligible=1` only for an admitted declaration; do not let a caller set `execution_mode` directly.

### Testing stage

- [x] Run `python -m unittest tools.session_coordinator.tests.test_resource_budget tools.session_coordinator.tests.test_cargo_reservations tools.session_coordinator.tests.test_database` after M1 implementation. Expected result: all selected tests pass, including migration of a pre-48 database and rejection of a burst declaration for `cargo test`.
- [x] Run `git diff --check -- tools/session_coordinator/resource_budget.py tools/session_coordinator/cargo_jobs.py tools/session_coordinator/cargo_reservations.py tools/session_coordinator/migrations.py tools/session_coordinator/cli.py tools/session_coordinator/tests`.

## M2 — Start-time burst admission, process supervision, and cleanup

### Implementation slices

- [x] Add `CargoJobService._choose_cpu_execution_mode(reservation, *, session_id, lane_scope)` as a thin adapter around `resource_budget.WindowsResourceProbe` and `cpu_burst.select_cpu_burst`. It always returns warm for a non-eligible reservation. For an eligible pending reservation behind a running warm reservation, it calls `burst_decision`, chooses burst only when eligible, and otherwise returns warm with the current denial reason. The warm head remains FIFO: a burst never consumes or changes a warm reservation’s queue position. Persist only the selected `execution_mode`; do not persist momentary CPU, memory, or disk reasons.

- [x] In `consume_cpu_reservation`, bind a burst reservation only after `_choose_cpu_execution_mode` returns `burst`; derive its exact target as:

  ```python
  burst_target = self.target_policy.validate(
      Path("E:/cargo-targets/zircon-engine/burst") / reservation_id
  )
  ```

  Require that the resolved target has `E:\cargo-targets\zircon-engine\burst\` as its canonical prefix and is different from every active Cargo job target. Set the resulting job cleanup policy to `delete_on_release`; keep warm jobs on their compatibility-selected reusable target and retained policy. If target validation, resource sampling, or directory creation fails, return the normal warm FIFO denial and leave the pending reservation unchanged.

- [x] Update the start, heartbeat, finish, release, and orphan reconciliation paths so an active burst reservation is process-supervised exactly like warm work. On release, schedule only its isolated target for existing asynchronous cleanup; do not invoke `cargo clean` on a warm target and do not wait for cleanup before releasing the Session’s other work.

- [x] Preserve all existing command-fingerprint and source-manifest checks for both modes. A source change after reserve must reject either warm or burst consume before Cargo starts.

### Testing stage

- [x] Run `python -m unittest tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cargo_reservations tools.session_coordinator.tests.test_maintenance_cpu_reservation_consume`. Expected result: all selected tests pass, including (1) a headroom-qualified `cargo check` burst starts in an `E:` target while warm work remains running, (2) `cargo test` stays FIFO-warm, (3) a second burst is denied, (4) low disk/CPU/memory retains FIFO-warm without consuming the reservation, and (5) release deletes only the burst target.
- [ ] Run `python -m tools.session_coordinator --json maintenance tick` against a fixture state after the test suite; expected result: no warm target is listed as deleted and terminal burst cleanup is reported through normal cleanup state.

## M3 — Explicit WIP and operator-facing flow visibility

### Implementation slices

- [x] Extend the snapshot reservation row with exact fields `executionMode` and `burstEligible`. The query must expose no command, absolute target path, environment, compatibility payload, source-manifest content, or instantaneous resource sample. Add a separate `cpuBurst` object under `validation` with `{ capacity: 1, active: 0|1, eligiblePending: number }`, calculated from the same SQLite snapshot and bounded to one object. Resource admission remains at consume time so page refreshes cannot block on CPU sampling.

- [x] Extend TypeScript contracts and validators with:

  ```ts
  executionMode: "warm" | "burst";
  burstEligible: boolean;
  cpuBurst: { capacity: 1; active: 0 | 1; eligiblePending: number };
  ```

  Preserve rolling-upgrade behavior by supplying the existing warm-only default when `cpuBurst` is absent from an older daemon.

- [x] In `OverviewPage.tsx`, render `CPU 热缓存 WIP` and `CPU 突发 WIP` separately. The warm panel states that it owns the reusable cache; the burst panel states its exact capacity, active count, and number of declared low-interference candidates. In `ValidationPage.tsx`, label each reservation `热缓存` or `隔离突发`, and show `可隔离检查` for an eligible warm pending reservation; preserve the current rule that a running reservation uses process-health wording instead of expiry. Both pages must repeat that these resource limits never close Session admission.

- [x] Update `validation-queue.md` with the four burst admission predicates, the `cargo check`-only declaration, exact cleanup policy, and the fact that a red/unavailable burst indicator is capacity information rather than a request to stop or drain work.

### Testing stage

- [x] Run `python -m unittest tools.session_coordinator.tests.test_control_snapshot` and `npm --prefix tools/session_coordinator/web run check`. Expected result: snapshot order remains bounded, malformed burst fields are rejected, 53-or-more browser tests pass, Vite builds, and `verify-dist.mjs` verifies every hashed asset.
- [x] With a coordinator fixture containing one warm running reservation, one burst running reservation, and two pending warm reservations, verify the production snapshot shows warm and burst WIP separately while all listed Sessions remain executable.
- [x] Run `git diff --check -- tools/session_coordinator/control_plane/snapshot.py tools/session_coordinator/web docs/tools/session_coordinator/validation-queue.md docs/superpowers/plans/2026-07-17-coordinator-adaptive-cpu-burst-lanes.md`.

## M4 — Controlled production rollout and acceptance

### Implementation slices

- [x] Load the migration only through a no-live-Cargo service rollover. Do not use drain, force-stop, process termination, or manual SQLite changes. Confirm the successor descriptor advances the schema and remains `read_write` before accepting traffic.
- [ ] Keep default callers warm-only for the first observation window. Enable `--burst-eligible` only for one explicitly requested package-scoped `cargo check` after a live snapshot reports an available burst slot. Do not create a burst target for a test, workspace build, GPU command, or a drive below the 100 GiB threshold.
- [ ] Observe the first burst through its start, process health, completion, release, and asynchronous deletion. Confirm the warm job and ordinary Sessions continued uninterrupted and that its target no longer exists after cleanup.

### Testing stage

- [x] Run `python -m tools.session_coordinator --json status`, `python -m tools.session_coordinator --json cargo list`, and `Invoke-RestMethod http://127.0.0.1:6518/control/v1/snapshot` after rollout. Expected result: one warm WIP, at most one burst WIP, no global maintenance hold, and queue rows expose exact mode/reason fields.
- [x] Run `npm --prefix tools/session_coordinator/web run check` if the production build was not already produced in M3, then verify `/ui/` returns the generated index and the current Validation/Overview chunks contain `隔离突发` and `Session 准入`.
- [ ] Update `docs/tools/session_coordinator/validation-queue.md` with the observed accepted behavior, then append one accepted M4 record only to this plan’s status table. Do not add a concrete record to `index.md` or `.codex/sessions`.

## M5 — Session liveness without resource retention

### Implementation slices

- [x] Raise the default business Session liveness window from 600 to 3600 seconds. Keep the 300-second lease plus 120-second grace and every Cargo reservation TTL unchanged, so a missed business heartbeat no longer expires unrelated work while abandoned resource ownership still resolves promptly.
- [x] Document that Codex-source presence remains telemetry-only: it cannot silently obtain business write authority, renew a foreign lease, or change Cargo admission.

### Testing stage

- [x] Run `python -m unittest tools.session_coordinator.tests.test_sessions`; expected result: the 3600-second default is asserted alongside unchanged lease and grace values, and stale/terminal reservation behavior remains covered.
- [x] Run `git diff --check -- tools/session_coordinator/config.py tools/session_coordinator/tests/test_sessions.py docs/tools/session_coordinator/control-plane.md`.
- [x] Roll the new default into production only through the next verified no-Cargo rollover, then confirm the successor is `read_write` and uses `session_ttl_seconds=3600` without changing active lease or reservation counts.

## M6 — Running validation retains Session executability

### Implementation slices

- [x] Treat a managed `cargo_jobs.status='running'` row as direct work evidence during the Session stale sweep. Guard both candidate selection and the conditional stale update, so a job that starts during the sweep cannot be invalidated by a stale candidate read.
- [x] Keep `leased` jobs eligible for normal stale recovery. Do not extend the owner heartbeat, file lease, CPU reservation TTL, cleanup policy, or any warm/burst FIFO position.

### Testing stage

- [x] Run the focused `test_mark_stale_preserves_reservation_bound_to_running_job`: an old-heartbeat Session remains active while its managed job runs, then becomes stale after the job finishes and releases.
- [x] Roll the corrected stale policy into production only through a verified no-Cargo `service.rollover`; confirm the successor remains `read_write` with no maintenance hold or hidden drain.

## M7 — Safe validation-lane projection

### Implementation slices

- [x] Replace raw `cargo_jobs` serialization in both browser validation arrays with one narrow lane projection containing only ownership, lane/state, lifecycle timestamps, and cleanup policy/status. Keep target existence checks and artifact lifecycle aggregation server-side.
- [x] Replace the browser table's command/path/PID-oriented columns with compact lane, state, duration, artifact policy, and cleanup state columns. Preserve rolling-upgrade parsing of an older daemon's supersets without allowing the new table to render those fields.

### Testing stage

- [x] Run the focused snapshot and web package validations. Prove exact safe server-side keys, no raw field rendering when an unexpected legacy payload is supplied, TypeScript compatibility, production build, and hashed asset verification.
- [x] Roll the safe projection through a verified no-Cargo `service.rollover`; confirm `read_write`, open Session admission, and live `cargoJobs` plus current-target rows contain exactly the safe lane fields.

## M8 — Current sync health before historical trend

### Implementation slices

- [x] Make the Overview's primary sync metric derive from the existing `codexSessions.lastRun`: unchanged success is `安静`, a visible lifecycle change is `+N`, and partial/failed/diagnostic/unavailable runs are `需关注`.
- [x] Add a compact sync panel that shows the last scan's scope and duration first, then retains the existing 24-hour quiet-run and visible-change totals as non-gating trend context.

### Testing stage

- [x] Run the full web check. Prove a quiet latest scan outranks a noisy historical aggregate, legacy snapshots display `未采样`, and typecheck, tests, build, and asset verification remain green.
- [x] Serve M8 from the current daemon's static distribution and verify the live Overview chunk contains the current-sync wording without changing Session admission. A daemon rollover is unnecessary because this is a static browser asset only.

## M10 — Rollover duplicate coalescing

### Implementation slices

- [x] Preserve the live incident evidence: two rollover actions succeeded 16 seconds apart because the second request arrived after the first successor marked itself healthy.
- [x] Add a narrowly scoped stabilization policy for the current healthy successor. A duplicate rollover returns an auditable no-op result naming the first action and successor; it does not create a second lifecycle intent or schedule another shutdown.
- [x] Keep the policy local to service rollover. It does not reject Session, lease, Cargo, validation, or normal later rollover work outside the short window.

### Testing stage

- [x] Run the full supervision-action regression suite. Expected result: a successor converts a duplicate rollover into `succeeded/coalesced`, retains `healthy` admission, and never invokes a second shutdown callback. Verified 2026-07-17: 31/31 passed.
- [x] Publish only after a naturally empty managed-Cargo window using `service.rollover`; verify the successor remains `read_write`, `maintenanceHold=false`, and the action history contains one physical successor handoff for concurrent requests. Verified 2026-07-17: `658bb4…` succeeded `3a172…`; the immediate duplicate `c9cf4…` returned `succeeded/coalesced` naming that action and the same successor, with no second shutdown.

## M11 — 验证等待期间的同里程碑续作与主任务回归

### Implementation slices

- [x] Preserve the root cause: the board labels a Session `waiting_validation`, but neither the snapshot nor the periodic observer turns the remaining implementation work in that Session's numbered plan into an actionable continuation.
- [x] Add a bounded, read-only continuation projection for `waiting_validation` and `waiting_lease` Sessions. It selects one unchecked implementation slice from the Session's own numbered plan, never consumes a Cargo reservation, mutates status, claims a foreign lease, or creates cross-plan WIP.
- [x] Include an explicit code-first rule: while the primary milestone still has a same-plan non-validation implementation/documentation slice, complete one such slice at a time and leave the validation in FIFO. After each slice, return to the primary milestone and select its next code slice; consume/interpret validation only after code candidates are exhausted or it becomes terminal.
- [x] Render the recommendation as a separate Kanban-like work-board panel, with the local wait reason, one same-plan candidate, the scope-claim reminder, and the main-task return rule.
- [x] Update the recurring coordinator observer so an active foreign validation is a cue to perform the bounded continuation instead of a cue to wait silently.

### Testing stage

- [x] Run snapshot and browser contract/component tests. Verified 2026-07-17: `test_control_snapshot` 13/13; front-end typecheck, 56/56 tests, production build, and 27 hashed asset checks passed. A waiting Session receives one bounded same-plan unchecked implementation candidate; testing-only checkboxes and untrusted/non-numbered plan paths never become browser work; active validation keeps code-first continuation guidance while terminal validation returns the primary milestone to the front.
- [ ] Roll out only through a natural no-Cargo `service.rollover`; then verify `read_write`, `maintenanceHold=false`, one queued validation continues to show as local-only, and the Overview exposes a continuation rather than a global stop.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1-M3 | 可选隔离 CPU 突发、控制面 WIP 与浏览器可视化 | accepted-local | 2026-07-17 | Python M1 54/54、M2 94/94、控制快照 40/40；前端 53/53、构建和 27 项资源校验通过；`git diff --check` 通过。M4 仅等待无 Cargo 生产窗口。 |
| M5 | 放宽业务 Session 失活、保留短资源 TTL | accepted-production | 2026-07-17 | `test_sessions` 14/14、前端 53/53、构建与 27 项资源校验、范围格式检查通过；无 Cargo rollover 后实时快照为 Schema 48、`read_write`、`sessionTtlSeconds=3600`，5 个 FIFO 预约保留。 |
| M6 | 运行中受管验证保持所属 Session 可执行 | accepted-production | 2026-07-17 | 定向回归先复现旧行为，再证明运行中不转 stale、终结后按原 TTL 回收；自然无 Cargo rollover 后新实例 `b35ffd…` 保持 `read_write`、`maintenanceHold=false`、`sessionTtlSeconds=3600`。 |
| M7 | 安全验证通道投影与紧凑实时表格 | accepted-production | 2026-07-17 | 快照回归 10/10；前端类型检查、53/53 测试、生产构建与 27 项资源校验通过；自然无 Cargo rollover 后实时 `cargoJobs`/`currentCargoTargets` 均为十个安全字段，验证页面块包含“Cargo 实时通道”且不含“托管命令”。 |
| M8 | 最近同步健康优先于历史噪声 | accepted-production | 2026-07-17 | 前端类型检查、53/53 测试、生产构建与 27 项资源校验通过；实时 Overview chunk 包含“最近一次安静同步”“24 小时趋势”，不含旧“静默同步”主指标；真实最近扫描为 245 项、0 变更、294ms，Session 准入仍为 `read_write`。 |
| M9 | 运行作业进程观察可视化 | accepted-production | 2026-07-17 | 红测先证明旧快照缺少结论字段；快照回归 11/11、前端类型检查、54/54 测试、生产构建与 27 项资源校验通过。无 Cargo 窗口的受控 `service.rollover` 后，继任实例 `b52be…` 为 `read_write`、`maintenanceHold=false`、Session TTL 3600；实时运行作业显示 `process_observation=observed`，浏览器行仅有 11 个安全字段，不含 PID、命令或路径。 |
| M10 | 重复 rollover 合并 | accepted-production | 2026-07-17 | 监督动作回归 31/31、控制快照回归 11/11、范围格式检查通过。自然空窗后后继 `658bb4…` 保持 `read_write`、`maintenanceHold=false`；受控重复请求 `c9cf4…` 成功返回 `coalesced`，指向唯一物理接班动作 `3a172…`，没有第二次关停。 |
