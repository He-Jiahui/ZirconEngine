# Runtime22 Fixed-Step Transaction Architecture And Performance Plan

- Date: 2026-08-24
- Source plan: `docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- Scope: `TIME-P0-002`, with prerequisite interfaces for `TIME-P1-014`, `TIME-P1-018`, and `TIME-P1-028`
- Status: fixed clock/debt transaction, committed interpolation, core outer-frame hard-cut, and typed failure receipt implemented; validation-copy requests submitted, but managed validation and the release performance matrix remain pending after pre-Cargo artifact-governance rejection; effect/state atomicity, committed fixed telemetry, Tracy/WPR capture, and final acceptance pending

## Current-Source Findings

The current implementation is structurally inconsistent with a committed fixed
simulation timeline:

1. The previous World path called `Time<Fixed>::drain_steps` before schedule
   execution. The current `WorldTimeController` now produces a non-mutating
   proposal; each `WorldDriver` fixed iteration begins and commits or aborts
   exactly one step.
2. The previous `WorldDriver` reconstructed step values by subtracting a final
   clock value. The current path supplies `SimulationTickId` and proposed time
   from the active capability, while `LevelSystem::world_time()` remains at the
   prior committed value until commit.
3. `SceneScheduleRunner` returns a `CoreError` from a failing runtime callback.
   `WorldDriver` retains one active step and aborts it on returned error, drop,
   unwind, or a detected World-generation change, so clock/debt are preserved;
   it still exposes no typed failure receipt, failing system classification, or
   common effect journal. Worker command buffers have a local discard path,
   but direct World mutation and external effects have no common step journal.
4. The independently pre-advanced core virtual/fixed clock and the global
   `time.fixed_steps` diagnostic have been hard-cut. `FrameTimeSnapshot` now
   contains only outer real-frame evidence, discontinuity, and the bounded
   fixed-step budget; every virtual/fixed observation is owned by a Level.
   Managed validation remains pending, so this is implemented, not accepted.

This is not a hot-loop micro-optimization problem. The central defect is that
the state machine has one `advance` transition where it needs separate
proposal, begin, commit, and abort transitions. Replacing the synthetic
context arithmetic alone would leave the future-state leak and failure
semantics unchanged.

## Reference Direction

`dev/bevy/crates/bevy_time/src/fixed.rs` advances its fixed clock once per
schedule iteration before that iteration runs. This establishes the useful
minimum invariant that each iteration observes one distinct fixed timestamp.

`dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp`
uses explicit `StartFrame`, tick-group release, blocking completion, and
`EndFrame` boundaries. Zircon should preserve that owner separation: a
World-local simulation tick owns its begin/commit lifecycle, while the outer
runtime owns only the monotonic frame input. It must not introduce a global
time manager or make render, task, and profiling clocks part of the simulation
transaction.

## Target State Machine

```text
FrameTimeSnapshot
        |
        v
WorldTimeController::accept_outer_frame
        |
        v
Fixed debt proposal (bounded by product policy)
        |
        +-- no available step --> regular stages and interpolation observation
        |
        v
begin_fixed_step -> SimulationTickContext -> fixed-stage execution
        |                                      |
        |                                      +-- failure --> abort receipt; debt unchanged
        v
commit_fixed_step -> committed fixed clock -> ordered outbox publication
```

The transaction object must be non-cloneable and bound to exactly one
`WorldGeneration`, fixed-domain epoch, and next tick index. It is the only
capability that may consume one timestep of debt. `SimulationTickId` must
contain `{ world_generation, fixed_epoch, tick_index }`; a naked `u64` is not
sufficient across world replacement or rate changes.

During an active step, the public committed clock stays at the previous
committed value. Fixed systems receive the proposed next delta and elapsed time
only through their typed `SimulationTickContext`. This prevents unrelated
consumers from observing an uncommitted future state. A successful commit
advances exactly one timestep, removes exactly one timestep of debt, and makes
the committed value observable. An abort leaves both committed clock and debt
unchanged and records the failed stage/system classification.

## Delivery Slices

1. Add a world-owned fixed-step proposal and non-cloneable begin/commit/abort
   authority. Remove `WorldTimeController::advance` as the path that drains a
   batch. Its snapshot becomes a frame proposal plus the last committed fixed
   state.
2. Replace `Option<u64>` in `SystemTickContext` with
   `Option<SimulationTickId>` by a hard cutover. Build fixed stage contexts
   from an active step, not by subtracting a pre-advanced final state.
3. Make `WorldDriver` begin one step, execute all three fixed stages, then
   commit that step. On a returned `CoreError`, abort only the active step and
   return a typed `FixedStepFailureReceipt` containing committed count,
   remaining debt, tick identity, and failing stage/system when known.
4. Add a tick-scoped effect journal/outbox before claiming full failure
   atomicity. Direct mutable World systems cannot be rolled back generically;
   therefore this slice may guarantee clock/debt atomicity first, but must not
   claim state/effect/replay atomicity until deferred mutations, external
   effects, RNG scopes, and snapshots use the same commit boundary.
5. Completed implementation: add `FixedInterpolationContext` from
   previous/current *committed* state and fractional actual remaining debt.
   It is read from `LevelSystem` or a runtime-system context, does not use a
   pre-advanced fixed elapsed value, and resets endpoint history on World
   replacement or fixed-timestep epoch change. This remains unaccepted until
   managed validation and profiling evidence exist.
6. Completed implementation: hard-cut the core compatibility fixed clock and
   its public virtual/fixed accessors after migrating its consumers to
   World-owned observations. `FrameTimeSnapshot` now carries outer-frame
   evidence and product budget only; it does not imply a globally advanced
   `WorldFixed` state or drive world diagnostics. This remains unaccepted
   until the managed validation and profiling evidence exists.

Policy mutation rejects while a fixed step is active. World replacement during
fixed execution is detected before another fixed stage or commit and aborts
the active transaction; the final generation check shares the World lock with
the commit so replacement cannot race the commit boundary. A fixed-timestep
change also rejects while debt is pending, preventing re-interpretation under
a new rate; virtual-only policy updates preserve the fixed epoch. The broader
rate-change migration contract remains `TIME-P1-017`.

## Core Clock Hard-Cut Research

### Consumer Map And Decision

The current implementation has already moved the executable simulation
timeline below the core boundary:

- `DefaultLevelManager::try_create_level(...)` reads `CoreHandle::time_policy()`
  exactly once and initializes the new `LevelSystem`'s `WorldTimeController`.
- `RuntimeDynamicSession::tick_frame(...)` passes only the outer
  `FrameTimeSnapshot` into the Level path; its fixed-step budget was admitted
  from the selected product policy during the core frame advance.
- `WorldDriver` derives virtual delta, pause, fixed debt, fixed plan, and
  fixed clock stamps from the Level-local controller. It does not consume the
  core `FixedStepPlan`.
- Product code outside `core` does not observe core virtual/fixed time. The
  former core/prelude tests, test SDK setup, physics fixture, and global
  `time.fixed_steps` diagnostic have been migrated or removed by this cut.

Consequently, `RuntimeTimeClocks { real, virtual, fixed }` is duplicate
mutable state. It has no valid shared-world interpretation: two Levels may
legitimately have different pause state, speed, fixed timestep, debt, and
commit count. Keeping it as a public observation surface would recreate the
old pre-committed timeline leak after the World transaction is fixed.

The hard-cut target is therefore:

```text
CoreRuntime
  RuntimeTimeAuthority (private)
    - Time<MonotonicReal>
    - default TimePolicy for subsequently created Levels
    - default-policy generation
  FrameTimeSnapshot
    - outer frame index, raw real delta, real clock stamp
    - accepted fixed-step budget and one discontinuity receipt

LevelSystem
  WorldTimeController
    - Virtual pause/scale/clamp, Fixed debt and committed clock
    - World-local policy generation and WorldVirtual/WorldFixed stamps
    - begin/commit/abort SimulationTick capability
```

`RuntimeTimeClocks`, `CoreRuntime::{virtual_time,fixed_time,pause_virtual_time,
unpause_virtual_time}`, core-owned virtual/fixed snapshot fields, and
`TIME_FIXED_STEPS_DIAGNOSTIC` are removed together. There is no deprecated
alias, re-export, or forwarding accessor. `TimePolicyReceipt` remains the
receipt for changing the default policy because it is a configuration
transaction, not a clock observation. Applying that policy after a Level is
published continues to affect only later Levels; broadcasting it to active
worlds is a separate `TIME-P1-017` transaction and must not be implied by this
cutover.

### Reference Review

`dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp`
binds a `UWorld`, frame delta, and tick type at `FTickTaskManager::StartFrame`,
releases bounded tick groups, then clears that World context in `EndFrame`.
The outer loop does not maintain a second globally committed world timeline.
`dev/bevy/crates/bevy_time/src/fixed.rs::run_fixed_main_schedule` similarly
accumulates virtual delta and expends fixed debt around the fixed schedule.
Zircon deliberately differs at the commit point: its Level transaction leaves
the public fixed clock unchanged until all three fixed stages succeed.

### Algorithm, Concurrency, And Measurement Contract

The removed architecture mutated three `Time` values in core and then mutated
a second virtual/fixed pair in every Level. The core fixed drain was `O(1)`
but semantically unused by scheduling; it wrote one misleading global
fixed-step diagnostic. Current source performs one core real-clock advance
plus one diagnostic-store acquisition, while each Level performs its own
`O(1)` proposal and each accepted fixed step performs `O(1)` begin/commit
bookkeeping. Committed interpolation reads are also `O(1)`: they retain two
tick identities and use actual fixed debt modulo timestep, without scanning
debt, cloning a World, or allocating on a no-step frame.

This is a structural reduction, not a completed performance result. The
managed validation/profiling lane is unavailable, so there are no timing,
allocation, mutex-wait, RSS, WPR, Tracy, or energy measurements. When it is
available, compare the old and new frame path with 0/1/8/capped steps and
1/100/1000 registered systems. Record core time-lock wait/hold time, Level
time-lock wait/hold time, fixed proposals, committed/aborted steps, diagnostic
write time, CPU time, allocations, and power outside `C:` under
`E:\\builds\\zircon-profile`. Acceptance requires the eliminated core fixed
work and diagnostic to be absent from the trace, not merely a lower average.

### Implementation Order And Risks

1. Completed: replace the core bundle with a private outer-frame/default-policy
   owner and shrink `FrameTimeSnapshot` to outer evidence plus budget.
2. Completed: remove the public core derived-clock API and migrate test
   fixtures to set a default policy before Level creation.
3. Completed: move virtual/fixed assertions to Level tests and keep frame
   diagnostics scoped to real time. World fixed telemetry remains pending on
   the committed receipt/outbox owner.
4. Completed: update core-time, prelude, and frame-schedule documentation in
   the same change. `docs/zircon_runtime/scene/ecs.md` is currently lock-owned
   and is intentionally not edited by this slice.

The only behavior-sensitive risk is callers that apply a policy after Level
creation and expect live propagation. Current production construction applies
the profile at lines 114-119 before its Level is created at lines 267-273; the
hard cut preserves that path. A later explicit broadcast/defer design must
carry every active Level's debt and cannot reuse the removed core clock.

## TDD Contract

The implementation begins with failing tests for these behaviors:

- Three accepted steps expose consecutive `SimulationTickId` and proposed
  elapsed values to all three fixed stages, while `world_time()` exposes only
  the prior committed state until each step commits.
- A failure in step `N` leaves exactly `N - 1` fixed steps committed and keeps
  the failed and later steps as measurable debt.
- Retrying the same input after an abort starts at the uncommitted tick index;
  no index, elapsed duration, or clock epoch is skipped.
- A max-step cap preserves all unconsumed debt and interpolation fraction is
  derived from that debt rather than clamped to a synthetic step count.
- Pause, policy mutation, world replacement, and schedule errors cannot commit
  an active or future tick accidentally; a pending-debt rate change is
  rejected and virtual-only configuration does not advance the fixed epoch.
- Worker command buffers discard uncommitted commands, and the eventual outbox
  test proves no external effect is published before commit.

## Profiling And Performance Evidence Plan

No execution-profile or power data exists for this change yet. The managed
Cargo validation lane is currently unavailable, so no local timing, allocation,
WPR, Tracy, or energy number is reported here. Static inspection establishes
only the current algorithmic shape: batch pre-advance is `O(1)`, fixed-stage
execution is `O(steps * stages)`, and the target transaction adds `O(1)`
begin/commit bookkeeping per executed step. It also adds two short
world-time-lock acquisitions per committed step instead of one pre-execution
advance per outer frame; the lock is deliberately released before systems run
so callbacks can inspect committed time without deadlock. The target must not
clone a World, scan outstanding debt, or allocate a journal on the no-step or
successful begin/commit paths.

When the managed lane is available, build profiling artifacts outside `C:` and
capture the same workload before and after the cutover:

```powershell
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
```

Record Chrome/Tracy scopes around fixed proposal, begin, each fixed stage,
commit, abort, and outbox publication. Add bounded counters for attempted,
committed, aborted, and deferred fixed steps; debt duration; active-step count;
and journal entries/bytes, plus world-time mutex acquisition/wait time. The
acceptance report must include 0/1/8/capped-step workloads, a failure at each
fixed stage, 1/100/1000 registered systems, and paused/hitch/resume cases. CPU
time, allocations, lock contention, RSS, and energy/WPR data are separate
measurements; no Unreal-comparison or power claim is valid until those captures
exist.

## Non-Goals For This Slice

- No dynamic runtime ABI exposure of replay controls.
- No global clock registry or UTC adapter.
- No promise of rollback for arbitrary direct World mutation.
- No compatibility shim that retains `drain_steps` as an execution authority.
- No C-drive build or profiling artifact output.

## 2026-08-29 Typed Failure Receipt Implementation

Runtime22 hard-cuts the Level tick error boundary from `CoreError` to
`LevelTickError`. A fixed-step failure now returns one
`FixedStepFailureReceipt` after the active clock transaction has aborted. The
receipt contains the exact `SimulationTickId`, failing stage, runtime system ID
when the scheduler can prove it, the count already committed in this outer
frame, actual remaining debt after abort, and the observed World generation.
Non-fixed failures remain the transparent Core branch.

`SceneScheduleRunner` retains the original `CoreError` and copies a runtime
system ID only inside the error closure. Worker-batch merge failures and World
generation checks remain deliberately unattributed instead of inventing a
system owner. The successful schedule path adds no receipt, journal, World
clone, debt scan, or system-ID allocation. `RuntimeDynamicSessionError` has a
dedicated Level tick branch so the receipt survives the dynamic-session
boundary instead of being flattened into a generic step string.

The RED contract added before production types existed requires a failure in
fixed tick 2 to report one committed step and 15 ms remaining debt, then retry
the same tick without an index gap. A second contract replaces the World during
`FixedUpdate` and requires the old tick generation, new observed generation,
zero committed steps, 10 ms debt, and no fabricated system ID. Upward
verification reconstructs the typed dynamic-session source and retrieves the
same receipt without parsing display text.

Managed validation is submitted as batches rather than per test. The initial
jobs `0b4e88a179db418e882cf48546512da0` and
`4be27bf6bda842659cff3808243e3dba` terminated during closure planning before
compilation because the external `zr_vm` path dependency had no descriptor.
The replacement jobs pin that repository at commit
`b730b40d6f85fcf5d28e590681213cc512704524` and do not absorb its dirty working
tree:

- full release library regression: validation-copy job
  `ce93984fad2e43c88e063b08581bfae6`;
- fixed-step functional, receipt, retry, and performance matrix: validation-copy
  job `ac7ae0d2fac54a09a541c7f7f9d0f28e`.

The release matrix runs 10 warmups plus 50 measured samples for every cross of
`0/1/8/capped` fixed steps and `1/100/1000` registered fixed systems. Schedule
registration is outside the measured interval; each sample verifies the exact
callback count before contributing to P50/P95. Timing results remain pending
until the managed job reaches a terminal result. Tracy, WPR, allocation-hook,
RSS, lock-contention, and energy claims remain outside this evidence.

This slice guarantees fixed clock/debt transaction evidence only. Arbitrary
direct World mutation and external effects still require the planned common
journal/outbox before state, effect, deterministic replay, or full rollback
atomicity can be claimed.

### 2026-08-29 Independent Review Correction

The first independent review found one important replacement-boundary defect.
`LevelSystem::run_runtime_scene_system` took a runtime system from the old
World, allowed the callback to replace that World, and then restored the old
system into the replacement World's schedule. Clock abort evidence remained
correct, but the new schedule could retain a closure and identifier that never
belonged to it.

The corrected path takes the runtime system and captures the replacement epoch
under the same World lock. After the callback, it restores the system only
through `with_world_mut_if_replacement_epoch`; an epoch mismatch drops the
retired system instead of injecting it into the current World. This preserves
the existing World-lock-to-epoch observation order and adds no compatibility
or fallback surface.

The World-replacement regression now proves all of the following in one
deterministic scenario:

- the callback runs exactly once before replacement;
- the replacement schedule does not contain the retired runtime system ID;
- the failed transaction retains exactly 10 ms of fixed debt;
- a zero-real-delta retry commits that debt in the replacement World; and
- the retired callback is not invoked by the retry.

At the original replacement-boundary review checkpoint, the source hashes were
`level_system.rs=f3c81222aec50888f68318cca080c53e85f4e5b1172084bb4d1131d6a80ce0c1`
and
`fixed_update.rs=fbdcac3798ef76bf063a93a3d858ddd2c42a2af2d0733ef2ad55b814634c4a48`.
Those values are historical baseline evidence, not current-tree attribution.
`git diff --check` and the structural checks for atomic take/epoch capture,
conditional restore, absent unconditional restore, clean replacement schedule,
and non-reinvocation are green.

Fresh post-correction managed validation is batched in validation-copy request
`dc8d485685aa4bc49237a98d5b1033bf`, job
`929bce90ab5d4f1f99a84f24c3510993`. It runs the release `core-min`
`fixed_step_` functional tests and ignored profiling matrix together. Dynamic
timing and pass/fail results remain pending until that asynchronous job reaches
a terminal result; they are not inferred from the structural checks.

The current batch layout gives all Runtime22 release gates the
`runtime22_performance_` prefix. One filtered release command therefore runs the
fixed transaction matrix, indexed runtime-slot comparison, and immutable outer
snapshot projection without recompiling once per benchmark.

The independent incremental review verified the two post-correction hashes and
returned `C0/I0/M0`; the original replacement-boundary finding is closed. It
confirmed that take-plus-epoch capture and compare-plus-restore each occur under
the World lock, that no reverse lock order was added, and that dropping the
unexecuted `FnOnce` closure retires rather than relocates the old system.

Coverage remains intentionally bounded. The dynamic-session unit test proves
that a typed Level error retains its receipt, while the compiled production
route in `state.rs` maps the actual Level tick result directly to the dedicated
Level branch. It does not fabricate a real-session fixed failure through a
wall-clock-dependent project fixture. A deterministic fixed-runtime panic
regression now crosses `WorldDriver`, `SceneScheduleRunner`, and `LevelSystem`:
the first panic aborts without advancing the committed clock, retains one full
timestep of debt, restores the runtime callback slot, and a zero-delta outer
frame retries and commits the same tick. This remains implementation evidence
until the managed batch accepts it. Commit rejection, worker-buffer merge
failure, allocation hooks, mutex wait/hold capture, and pause/hitch/resume
profiling remain explicit follow-up evidence rather than claims of this slice.

### 2026-08-29 Runtime Callback Slot Indexing

The fixed-step review also exposed an independent runtime callback hot path:
`SceneSystemRegistry` linearly removed a runtime closure from a sorted vector,
then restored it with a sorted insertion, while `Schedule` allocated a copied
ID and linearly scanned in-flight IDs for every callback. Runtime22 now owns a
follow-up slice recorded in
`2026-08-29-indexed-runtime-system-slots.md`: stable sorted slots retain the
ID/stage/order key, an ID-to-slot map provides constant-time take/restore, and
the schedule stores only an in-flight count. Cold registration/removal updates
shifted indices; native-system storage and topological plan ordering are
unchanged. This follow-up has not yet been committed or performance-accepted.

### 2026-08-29 Current-Tree Batch Classification

Coordinator job `83b23efd72734739bca23b5ffc494ac2`, run
`a19e6be2a6094f56b5b671458268c18a`, compiled the shared current tree in release
mode but exited `101` before running libtest. Its log contains 242 compile
errors rooted primarily in foreign in-progress asset/resource/test relocation
and unresolved cross-session surfaces, so it supplies neither functional nor
performance acceptance. Runtime22 repaired the two owned defects identified by
that batch: `Schedule` again retains the native taken-ID removal helper, and the
fixed interpolation regression reads the public Level interpolation snapshot.
The fixed-step matrix and indexed-slot paired gate must now pass in one
exact-manifest coordinator validation copy before this milestone can commit.

Current incremental source attribution for the paired handoff is
`level_system.rs=ccbe7e94fe014b6f57d3d004c88fd47b6dada3bc0014084e31f2d00d3400c5e8`,
`fixed_update.rs=03d4eeb1f4365570999baa894420091421f2daa7e208b4095c549d16c7025c3b`,
`schedule_stage_plan.rs=e393d65e24bfbd254630d7beb7d7a1c7ced6cedbf0340069668c814fddb931ff`,
`scene_system_registry.rs=30996055bcb3e7502d8b08f0504ed3f01da56202f7dcaba197ece65c31b6ddf4`,
and
`scene_system_registry/runtime_system_slots.rs=eff4059c03d557cf591713b980523580365c901511c8ccf66dbf62820511905b`.

The paired immutable-copy requests for this snapshot (jobs
`e7144133050547cd958b6aee6a7e186f` and `556e7112dc9b447e8fc1e39c573adaf8`)
were rejected by coordinator artifact governance before Cargo admission
because unmanaged foreign paths were present (`D:\\ZirconBuilds\\mvp-test-fixtures-35880`
and `E:\\ZirconBuilds\\editor08-validation-20260829`). A later paired retry
(`4bc65ff5f2d2408fa4f09ef3b97e63e9` and `bb72404894cd4fe28f53931dc0958521`)
was rejected at the same pre-Cargo stage for
`D:\\ZirconBuilds\\mvp-test-fixtures-35880`. No Runtime22 path was removed or
incorporated, and all rejected attempts produce no test or timing evidence.
Managed exact-manifest validation is still pending.

## Current batched validation handoff (2026-08-30)

After the Runtime22 source/static corrections, one coordinator-managed release batch was submitted
for the exact attributed Runtime22 overlay:

- request `0de1d97e5169482aa8df6b7e6b82b97d`;
- ticket `5eb188c3255e49749625da2e1b45473b`;
- command `cargo +1.94.1 test -p zircon_runtime --lib --release --locked --jobs 1
  --no-default-features --features core-min runtime22_performance_ -- --include-ignored --nocapture
  --test-threads=1`;
- source manifest hash `e746330c7b9fe14fe332f9e6e7ef8743f899beac0ac2cc7401d236061f5789b4`.

The batch covers fixed-step typed failure/retry, immutable outer-frame snapshot consumption, random
checkpoint/restore/replay, indexed runtime slots, schedule-stage allocation removal, and callback
panic recovery in one Cargo invocation. The receipt is currently `queued`; no compile, test count,
timing, or performance threshold result is claimed until the coordinator records a terminal result.

## Runtime22 checkpoint regression strengthening (2026-08-30)

The deterministic checkpoint/reseed regression now observes reseed's actual entry into the
registry-to-seed critical section before releasing the checkpoint hook. The fixed implementation
keeps that entry signal and completion absent while capture is paused; after resume the worker
enters, completes, and the test verifies the restored stream draw and post-reseed generation. The
batched managed validation remains queued; no Cargo result or performance acceptance is claimed here.

The regression has explicit entered and completion channels. Against the old implementation the
entered signal would arrive during the pause, exposing the mixed-era interleaving; against the fixed
implementation both signals remain absent until resume. This closes the earlier test weakness where
a start signal alone could race ahead of the actual reseed call.

The first managed batch (`5eb188c3255e49749625da2e1b45473b`) reached terminal failure because its
manifest contained the pre-strengthening `service.rs` hash. A replacement batch was submitted with
request `48931890ab784ae6bd790a7a79609c80`, ticket `691977ae5d44431c8f4540b25057cb65`, and current
source manifest hash `7a7b71e9b905f1c5dd318d55fffc2dfbd8398b0157b52dfbf02d4fc6a475b122`. It is queued;
compile, test, timing, and performance results remain unclaimed.

After adding the explicit entered/completion barriers, that replacement manifest was superseded
before execution. The current queued batch uses request `2f05decb30214d56944117e8fd14f4a5`, ticket
`a92cb16915a146dc87c64e3b1585c95c`, and source manifest hash
`da141b434e5240397916b41b0a50e48bec0a303a497077c274d1bf30ddd14569`. The earlier queued ticket
`691977ae5d44431c8f4540b25057cb65` is retained as stale evidence only; neither ticket supplies a
Cargo, test, timing, or performance result yet.
