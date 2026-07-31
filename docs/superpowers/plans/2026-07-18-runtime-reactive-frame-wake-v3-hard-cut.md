# Runtime Reactive Frame Wake V3 Hard-Cut Implementation Plan

> **Execution:** Follow the approved design in `docs/superpowers/specs/2026-07-18-runtime-reactive-frame-wake-v3-hard-cut-design.md`. Use dependency-ordered owner sessions and coordinator-managed Windows Cargo only.

**Goal:** Close the Runtime03 reactive-frame producer gap with a V3-only dynamic ABI, session-safe async wake delivery, explicit next-frame demand, and product/WPR evidence.

**Architecture:** Runtime10 owns the V3 ABI and destroy quiescence barrier. Runtime11 owns a generic exactly-once terminal observer. Runtime03 aggregates synchronous frame demand and connects real animation/timer/visible-task producers to the host cadence. No compatibility surface is retained.

## Milestone M1: Runtime11 terminal observer prerequisite

### Implementation

- [x] Add red tests for observer registration before/after completion, exactly-once delivery, multiple observers, panic containment, dependency completion, and reentrant handle access.
- [x] Implement `JobHandle::on_terminal` outside the job-state lock and preserve dependency continuation behavior.
- [x] Add observer diagnostics only where they describe task ownership; do not add a scheduler-wide frame wake.
- [x] Update Runtime11 job-system documentation and structure/source guards.

### Acceptance

- [ ] Focused task tests and Runtime11 diagnostics tests pass.
- [x] No winit, dynamic API, or application policy dependency enters Runtime11.
- [x] Independent review reports Critical 0 / Important 0.
- [ ] Coordinator exact milestone commit returned before M2 begins.

## Milestone M2: Atomic Runtime10/Runtime03 V3 hard cut

This milestone is one indivisible source and commit boundary. V3 export/loader, session config, tick demand, host proxy registry, every production caller, and the first real producers land together. A constant-Idle V3, optional wake sink, temporary V2 path, or commit with unmigrated callers is forbidden.

### Implementation

- [x] Add red interface tests for V3-only symbol/table, raw `u32` demand-kind constants plus checked conversion, `ZrRuntimeSessionConfigV2`, wake-sink validity, checked delay handling, and retired V2 identifiers.
- [x] Add red lifecycle tests proving closing rejects late session calls and destroy waits for both active ABI action guards and in-flight wake callbacks.
- [x] Add red host tests proving the callback trampoline catches Rust panic before it can cross `extern "C"`, and token removal happens only after successful destroy.
- [x] Add red demand tests for per-tick idle, immediate dominance, shortest same-tick delay, timer cancellation, and replacement of the previous runtime deadline.
- [x] Implement folder-backed V3 ABI DTO owners, runtime export, V3-only app loader, session-slot lifecycle barrier, and session wake registration.
- [x] Create the event loop/proxy registry before runtime session creation and migrate every runtime app/session call site to V3 in this same milestone.
- [ ] Return the real `ZrRuntimeFrameDemandV1` from tick and apply it in `RuntimeFrameCadence`; do not land a constant placeholder.
- [ ] Record active-animation demand during the existing animation scan without a second world scan.
- [ ] Route actual runtime UI timer deadlines into the same-tick demand accumulator.
- [ ] Attach the M1 observer only to concrete frame-visible async result owners and wake on terminal state.
- [x] Keep Game/Continuous/Mobile Poll and Headless stable fixed deadlines unchanged unless an explicit product contract requires an earlier wake.
- [ ] Update Runtime10 and Runtime03 architecture docs, ABI inventories, source guards, and failure mirrors without returning Runtime03 fixed yet.

### Acceptance

- [ ] V2 symbol/table/loader identifiers are absent from production and no fallback exists.
- [ ] `zircon_runtime_interface` ABI/version tests pass.
- [ ] Runtime dynamic API lifecycle, demand, animation, timer, and visible-task wake tests pass.
- [ ] `zircon_app` V3 loader/session/cadence/source-guard tests pass with raw target-test counts.
- [ ] Concurrent destroy proves no ABI action or callback remains live after return.
- [ ] Independent review reports Critical 0 / Important 0.
- [ ] Coordinator exact milestone commit returned before M3 begins.

## Milestone M3: Runtime03 product and WPR closeout

### Implementation

- [ ] Write `docs/plans/zircon_runtime/runtime/03/2026-07-18-desktop-idle-cadence-wpr-budget.md` before capture with the exact command, machine facts, artifact paths, and fixed thresholds from the approved design.
- [ ] Run the Desktop idle product for a 5-second warmup plus 30-second measured WPR window.
- [ ] Parse ETL and cadence logs into retained CPU, wakeup, frame-pump, host-drain, and redraw artifacts.
- [ ] Run the same build in continuous Game mode and record median frame throughput.
- [ ] Run active-animation, delayed-timer, visible-task completion, coalescing, redraw-no-feedback, and shutdown-race product scripts.
- [ ] Update Runtime03 module docs and return the open failure only after every acceptance item is green.

### Acceptance

- [ ] After warmup, idle frame-pump/host-drain/redraw deltas are zero.
- [ ] Runtime process sampled CPU is at most 1.0% of one logical core and event-loop wakeups are at most 2 per second.
- [ ] Continuous Game median frame throughput loses no more than 2% against the recorded baseline.
- [ ] Raw ETL, parsed counters, product logs, command lines, source manifest, and machine facts are retained.
- [ ] Independent review reports Critical 0 / Important 0.
- [ ] Runtime03 failure has canonical failure-to-fixed return and coordinator exact milestone commit.

## Milestone M4: Cross-plan parity and status

- [ ] Run Runtime10 dynamic API/interface parity gates.
- [ ] Run Runtime11 task-model parity gates.
- [ ] Run Runtime03 schedule/frame-loop mirror and plan-status gates.
- [ ] Run the narrow required `zircon_runtime_interface`, `zircon_runtime`, and `zircon_app` Windows checks through the coordinator.
- [ ] Update numbered plan status/completed-item tables with exact jobs, raw counts, review counts, manifests, and commit SHAs.
- [ ] Keep the umbrella runtime goal open for unrelated Runtime02-15 work.

## Status And Completed Items

| Milestone | State | Completed evidence | Remaining |
|---|---|---|---|
| M1 Runtime11 observer | `implemented_static_pending_managed_validation_and_commit` | `JobHandle::on_terminal`, exactly-once/panic/reentrant/dependency tests, behavior inventory 26, Python audit 1/1, independent review C0/I0/M0 | focused Runtime11 Cargo and coordinator exact milestone commit |
| M2 atomic V3 hard cut | `in_progress_static_not_atomic_acceptance` | V3-only interface/export/session barrier/demand accumulator; app/editor V3 callers; host token registry and cadence; production V2 hits 0; app and dynamic lifecycle reviews C0/I0/M0 | real animation/timer/visible-task producers, runtime-absorption V3 mirrors, managed Cargo, atomic review and commit |
| M3 Runtime03 product closeout | `pending_blocked_by_M1_M2` | cadence state machine implemented; static audit 3/3; Headless review finding closed | V3 dependency, product tests, WPR, fixed return |
| M4 parity/status | `pending` | no claim | all cross-plan gates and status mirrors |
