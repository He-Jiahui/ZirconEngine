---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: task-terminal-delivery-bounded-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/core/runtime/tasks/callback_dispatcher.rs
tests:
  - managed zircon_runtime tasks filter
  - isolated 1/100/10000/100000 panic-dependency chain matrix
  - 1/100/10000 terminal fan-out and 0/1/100ms observer matrix
  - same-deadline slow-callback timer isolation
---

# Runtime11: bounded task terminal delivery

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：`PERF-MVP-585` task terminal and timer callback delivery
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：dependency completion, terminal observers, and timer callback affinity are
  Runtime11 task-kernel responsibilities. Asset, scene, and editor consumers must not each
  create a private callback queue or worker.

## 失败现象与复现证据

`JobState::publish_terminal` currently runs every dependency continuation and terminal observer
directly on the thread that completes the source job. A panicked prerequisite calls
`mark_panicked` on its dependent from that continuation, so a long panic-propagation chain is
recursive. A wide fan-out or a slow observer likewise monopolizes the completion worker.

`TaskTimer::run_timer` removes all due registrations then invokes each callback on its single
control-plane thread. One slow callback therefore delays unrelated due registrations. Existing
timer capacity limits registrations to 512 but do not bound callback execution time.

The current source already records this as `PERF-MVP-585` in the originating performance plan
and its 2026-07-17/2026-07-30 task-system review. This handoff makes the lower-layer repair and
its dynamic proof explicit. No dynamic before/after measurement is claimed yet: the 100000-depth
pre-change reproduction must run in an isolated child because a stack overflow can terminate the
test process.

## 最低共享层根因

Unreal Tasks is the primary system-scale reference: task prerequisites are scheduler work rather
than recursive user-stack calls, and its task tests include stress/fan-out coverage. Bevy is the
Rust landing-zone reference: it uses explicit task-pool ownership and cooperatively ticks work;
it does not create a callback OS thread per consumer.

Runtime11 will add one folder-backed `TaskCallbackDispatcher` owned by an existing `TaskPool`.
Terminal delivery and timer expiry only move callback ownership into FIFO envelopes; they do not
execute arbitrary consumer code inline. Each dispatcher run processes at most a fixed global
budget and a smaller per-envelope quantum, then requeues unfinished envelopes. At most two
existing pool workers deliver different envelopes concurrently, while every origin envelope
remains serial. This gives:

- producer cost `O(1)` beyond moving the callback collection already retained by a `JobState`;
- delivery cost `O(C)` for `C` callbacks, with bounded native-stack depth and no recursive
  prerequisite propagation;
- FIFO fairness between envelopes while preserving registration order within one origin handle;
- terminal state and `wait()` visibility before delivery, while observers begin only after that
  origin's dependency continuations are delivered;
- timer cancellation rechecked by the dispatched callback, so cancelling after a deadline is
  observed but before delivery still suppresses execution.
- a periodic timer registration has at most one admitted or running delivery; ticks that arrive
  while it is pending are coalesced, retaining the existing slow-callback back-pressure behavior
  without holding the timer control thread.

This is a deliberate hardening of `JobHandle::on_terminal`: registration admits a one-shot
observer for asynchronous delivery instead of running an arbitrary late observer synchronously
on the registering thread. Observer and continuation panics are contained so later callbacks
continue; task panic state remains the state recorded when the task itself terminated.

Before modifying the delivery algorithm, add isolated regressions and collect these measurements
from the exact built test binary on Windows, with artifacts only under `D:\ZirconBuilds`:

| Matrix | Measurements | Acceptance signal |
| --- | --- | --- |
| panic dependency chain: 1, 100, 10000, 100000 | child exit, all terminal states, process wall time, peak working set | no stack overflow or hang; all descendants reach the propagated terminal state |
| terminal fan-out: 1, 100, 10000 | callback count/order, dispatcher run count, maximum callbacks per run, queue drain time | exact once/order; no run exceeds the configured budget |
| observer latency: 0, 1, 100 ms | unrelated scheduled task start, completion-worker wall, callback queue age | observer work is not run inline by source completion; unrelated work progresses |
| same-deadline timer callbacks | healthy callback deadline lateness while one callback stalls | timer control thread remains available to publish other due callbacks |

Use WPR CPU/context-switch sampling when host policy permits. If Windows host policy rejects WPR,
record the exact error, retain process samples plus dispatcher counters, and make no CPU-stack,
power, or cross-engine performance claim. Compare only equivalent post-fix matrix runs; the
pre-change 100000 panic chain is a crash-safety baseline, not a throughput baseline.

## 架构修复验收

- `JobHandle`, `JobScheduler`, and `TaskTimer` share the same domain-neutral dispatcher contract;
  no consumer-specific queue, thread, compatibility path, or direct callback bypass remains.
- Default standalone handles and timers reuse one process dispatcher state; a scheduler still
  binds its dispatcher to its explicitly injected `TaskPool`.
- Repeated `JobScheduler::process_io()` construction reuses one process-I/O dispatcher state,
  so its two-runner budget is not multiplied per facade.
- A terminal producer performs no unbounded recursive callback delivery, and the timer thread
  performs no arbitrary callback body.
- Focused task, timer, panic-chain, fan-out, and observer ordering regressions pass through the
  managed Windows validator.
- The original Runtime11 `tasks` gate and the originating performance evidence are rerun with
  exact source manifests before this handoff returns as fixed.

## Validation Status

The first managed Windows `core-min` build reached the edited task module and failed with two
`E0365` visibility errors: `tasks::mod` tried to re-export dispatcher-private types to its
parent. The repair keeps the dispatcher private and changes only its sibling consumers to import
the module directly. The next managed build confirmed that repair: `Cargo build` passed.

The first managed lib-test compile was blocked before Runtime11 tests executed by `E0432` in
`zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs:9`; coordinator
ownership mapped that file to active Runtime74 session
`optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`. A later test attempt cleared that
external gate and reached Runtime11's private test module. It exposed a local `E0432`: the new
deep-chain regression imported `JobScheduler` and `TaskPool` through `job_handle` instead of the
parent `tasks` module. The same compile also exposed a local `E0061` in the pre-existing
`PendingScheduledJob` poison-lock regression after `pending_with_scheduler_diagnostics` gained its
dispatcher argument. Both repairs now bind the test fixture to the same explicit test pool; the
subsequent compile exposed two `E0277` timer-fixture errors because `std::sync::mpsc::Receiver`
is not `Sync` but timer callbacks require `Send + Sync`. Both slow-callback fixtures now use
`Arc<Barrier>`; the deep-chain test remains pending revalidation.

The next managed test request was rejected before Cargo started with
`unmanaged_artifacts_detected`. The coordinator listed pre-existing foreign target directories
under `E:\cargo-targets\` (including editor blend-space, shader, and pester runs); Runtime11 did
not create, own, delete, or adopt them. The task test must be rerun only after the coordinator's
responsible owner resolves that shared validation-environment gate.

The following retry acquired a fresh coordinator-managed `D:` test lane and reached Cargo, but
failed with `E0583` while another active Runtime74 session was changing
`zircon_runtime_interface/src/ui/binding/model/binding_value.rs`. Immediate read-only inspection
after Cargo exited found that file restored and modified under Runtime74's lease. Runtime11 did
not change that external file; this is a retryable concurrent-workspace compile blocker rather
than evidence about the task dispatcher. The deep-chain execution remains pending a clean
cross-plan compile snapshot.

After adding the test-only dispatcher budget metrics and the 10,000 fan-out regression, the
managed Windows command
`validate-matrix.ps1 -Package zircon_runtime -NoDefaultFeatures -Features core-min -SkipTest`
passed `Cargo build` in `7m16s` on the coordinator-managed `D:` target. That is a source build
check, not a delivery-performance result: the lib-test gate remains pending the Runtime11 test
compilation repairs above.

After the structural review added bounded two-runner delivery and periodic-tick coalescing, the
same managed `core-min` source build passed again in `6m00s`. This confirms the current production
sources compile, but does not exercise the test-only metrics or establish callback latency,
throughput, power, or cross-engine comparability.

The clean retry of
`panic_dependency_chain_uses_bounded_terminal_delivery` passed through the managed Windows
`core-min` lib-test lane. The end-to-end validator wall time was `705.3s`, including the lane's
first test-binary compile; it is not a callback-throughput measurement. The child process created
and terminally propagated the 100000-node panic dependency chain without stack overflow or hang.

The private `callback_dispatcher` lib-test group then passed through the same managed lane in
`546.3s` end-to-end. Its assertions cover 16-callback envelope rotation, per-callback panic
containment, the `64`-callback per-run budget, and a 10000-callback fan-out with exact once
delivery and `157` budgeted runs. The wall time remains compilation-inclusive and is not a
dispatcher throughput result.

The `timer::tests` group passed through that lane in `154.4s`. This exercises existing timer
behavior plus the two new regressions: a stalled same-deadline callback does not occupy the timer
control thread, and a stalled periodic callback leaves one pending dispatcher delivery rather
than accumulating one per elapsed interval. This is behavioral/back-pressure evidence only;
observer-isolation and profiling matrices remain pending.

The `job_handle` private-test group passed in `18.0s` on the warmed managed lane. It covers
dependency continuation terminal paths, observer delivery after a dependency continuation panic,
and the late-observer ordering boundary. This closes the private observer-isolation matrix;
profiling evidence remains pending.

Independent review then found four follow-up issues: terminal observers registered before
completion were split into independently runnable envelopes, repeated `process_io()` schedulers
multiplied their callback-runner budget, a timer could run an already queued callback after its
last owner dropped, and the envelope-rotation regression lacked an admission barrier. The repairs
retain dispatcher-budgeted delivery: one `JobState` now publishes terminal observers as an ordered
batch and releases the next late-observer batch only from the preceding batch completion; the
process I/O scheduler reuses a process I/O dispatcher; queued timer delivery upgrades a weak timer
owner and checks `closing`; and the rotation regression blocks the first callback before admitting
the later envelope.

The timer shutdown regression first failed as expected through the managed lane (`Cargo test`
exit `101`, `351.1s` including rebuild) before the closing-state repair. Its repaired rerun passed
in `393.4s`. The ordered-observer regression passed in `29.0s`, the process-I/O shared-budget
regression passed in `31.7s`, and the complete private `core::runtime::tasks` test group passed in
`25.8s` on the warm lane. These are validation wall times, not throughput, CPU, allocator, or
power measurements.

## 禁止临时方案

- Do not raise thread-stack size, lower the stress depth, or make a test-only recursive bypass.
- Do not add a private timer worker, editor worker, asset worker, unbounded channel, alias, or
  compatibility observer path.
- Do not silently drop terminal callbacks to impose an artificial queue cap.
- Do not report WPR, power, allocator, release-mode, or cross-engine results without the matching
  successful measurement evidence.

## 修复结果与回传

Open state: `resolving_failure`; the bounded dispatcher implementation and all owned private
regressions have passed managed Windows `core-min` validation. The existing cross-module observer
tests require an audited coordinator scope transfer for `zircon_runtime/src/tests/tasks.rs`; the
transfer preview is eligible but this session has no maintenance-capability authorization to apply
it. The Runtime11 structure audit also still has a 10-module owner inventory and therefore does
not classify the new private `callback_dispatcher.rs`; coordinator transfer preview
`015d9c48c2f5e84b6da70a21813358f20e73e517b5e548f9dc6bcc6b0870e056` found both required
inventory files eligible, but applying that transfer needs the same maintenance capability. WPR
is installed and idle, but no trace is started until its `D:` output has a coordinator-managed
artifact lifecycle. Therefore this record does not claim the originating cross-module gate,
structure-audit acceptance, CPU-stack profile, power result, release measurement, or cross-engine
comparison as passed.

The same Runtime11 audit has a 500-line owner-module threshold. The current root task modules
measure `job_handle.rs = 664` (`386` production lines before its private-test module) and
`timer.rs = 595` (`368` production lines before its private-test module); private regression
modules make the roots exceed the rule. The conforming repair is to move the private tests into
folder-backed test modules and update the guarded owner inventory, not to remove the regressions
or compact code artificially. Those new test paths and the audit inventory require the same
audited scope extension, so this session records the measured structural debt rather than
bypassing ownership.

The follow-up preview
`5341cd23f3985785f02b210e154c5c27b363cd392e8bb5eb665986cc19c8991b` confirms the two audit
inventories and `runtime_absorption/job_system/mirror_docs.rs` are eligible existing paths. The
proposed `tasks/job_handle/tests.rs` and `tasks/timer/tests.rs` are absent and therefore cannot be
acquired through an existing-path transfer. Creating those folder-backed owners needs a new
coordinator-authorized scope rather than a manual directory creation in this session.

On 2026-08-23, a successor registration requested the two new test paths together with the three
Runtime11 structure-audit inventories and mirror test. The coordinator accepted the request for
processing but did not produce a durable session; two recovery reads reported no successor. No
registration was replayed and no unscoped files were created. The only correct next action is a
coordinator-authorized successor or an explicit release/transfer from the active broad
`zircon_runtime/src` owner; this record remains `resolving_failure` until then.

Coordinator recovery later finalized both successor requests as
`plan_wip_limit_reached`: `runtime11-terminal-delivery-perf-m1-r1-01a019a5-20260823` is already
the executable primary for this plan family. The primary then submitted its complete prior scope
plus the two test paths and audit owners under request `61131c23c4084905b4b4370368ca8bd4`.
That request is recorded as `accepted` with no `completedAt`; it is not an authorization until the
coordinator persists the widened scope. No source, test, audit, commit, or notification action
may proceed from this pending receipt.
