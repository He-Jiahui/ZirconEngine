# Runtime22 Indexed Runtime-System Slots

- Date: 2026-08-29
- Source plan: `docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- Scope: replace runtime-system callback take/restore scans with stable slots and an ID index
- Status: implementation complete; managed batch validation and post-extraction review pending

## Finding

`SceneSystemRegistry::take_runtime_system` linearly scanned and removed a
sorted `Vec<BoxedRuntimeSceneSystem>`. Restoration performed another sorted
binary search followed by a vector insertion, and `Schedule` allocated a
temporary `String` plus linearly scanned taken IDs for every runtime callback.
The callback path therefore paid `O(N)` lookup/shift work for each system.

## Design

Runtime systems now live in stable sorted slots. Each slot retains its immutable
ID, stage, and order key while the closure is temporarily absent as
`Option<BoxedRuntimeSceneSystem>`. A `HashMap<String, usize>` maps IDs to slots.
Taking and restoring a callback use one index lookup and an option move; cold
registration/removal paths update sorted slots and shifted indices. The
schedule retains only a runtime in-flight count, while the registry index keeps
duplicate IDs rejected even while a callback is executing. Native-system
ordering and the compiled topological plan remain unchanged.

The slot/index container and its iterator are isolated in
`zircon_runtime/src/scene/ecs/scene_system_registry/runtime_system_slots.rs`.
`scene_system_registry.rs` remains the registration, policy-validation, and
schedule-graph owner; the extraction moves the slot behavior/performance tests
with their implementation and reduces that owner from 964 to 629 lines without
adding another callback lookup, allocation, or compatibility path.
Stage-plan counting, conflict-graph sizing, and executor-plan compilation now
consume the slot iterator directly instead of collecting a temporary
`Vec<&BoxedRuntimeSceneSystem>` on each schedule rebuild. `PlanNodes` borrows
the registry and materializes only the per-stage node vector required by
topological ordering.

## Deterministic Contract

- runtime callback take and restore do not scan or shift the runtime slot list;
- a taken runtime ID remains reserved and cannot be registered again;
- restoration returns the closure to its original deterministic stage/order slot;
- a deferred schedule rebuild still waits until both native and runtime in-flight
  counts reach zero;
- a panicking runtime callback restores its stable slot and in-flight count
  before the original panic resumes, so the same callback remains retryable;
- replacement-world retirement continues to drop a closure when its generation
  changes rather than injecting it into the replacement schedule.

## Validation And Performance Gate

The structural contract in
`zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs` asserts the
stable-slot/index architecture and rejects the old runtime ID vector path. The
registry behavior tests cover non-tail sorted insertion, multiple simultaneous
holes, reverse restoration, in-flight duplicate rejection, middle removal,
shifted-index rebuild, and reinsertion. A second behavior test creates a real
`A -> B -> A` schedule cycle, verifies failed registration removes B, and then
takes/restores/takes surviving A through its retained index. Independent review
returned `C0/I0/M0` after those tests closed the original coverage finding. The
semantic result remains evidence for the stable-slot design; the subsequent
module extraction still requires a fresh incremental review before closeout. The
existing ignored fixed-step release matrix remains the primary behavioral
batch, covering `0/1/8/capped` steps and `1/100/1000` registered systems with
10 warmups and 50 measured samples. A paired release-only gate additionally
executes the same ordered 1,000-system take/restore cycle through the exact
legacy vector scan/remove/sorted-insert algorithm and the indexed slot path.
It uses 10 warmup pairs and 31 measured pairs, alternates which path runs first,
reports both P50/P95 values, and requires indexed P95 to be at most 50% of
legacy P95. The test rejects binaries with debug assertions and emits
`profile=release`; all three Runtime22 release gates share one filter, and the exact performance
batch command is `cargo +1.94.1 test -p
zircon_runtime --lib --release --locked --jobs 1 --no-default-features
--features core-min runtime22_performance_ -- --include-ignored --nocapture
--test-threads=1`. The accepted result must include exact callback counts and
P50/P95 rows for every fixed-step cross plus the paired reduction row; no timing
result is inferred before the managed run reaches a terminal state.

The schedule compiler's topological ordering path was also audited during this
batch. Its per-stage `same_stage_by_id` map was populated but never read;
removing that dead allocation preserves the existing linear reference lookup
and topology ordering while reducing schedule rebuild work. The structural
suite now rejects the unused map from returning.

The measured lane must use coordinator-managed immutable-copy evidence. Foreign
compile-time resources and unrelated working-tree changes are not part of the
Runtime22 overlay and do not authorize absorbing graphics-owned files.

The current-tree release batch job
`83b23efd72734739bca23b5ffc494ac2`, run
`a19e6be2a6094f56b5b671458268c18a`, exited `101` before libtest execution.
The shared checkout produced 242 compile errors, beginning with missing foreign
asset/resource test files and unresolved cross-session crate surfaces; it
therefore produced no admissible timing row. Runtime22 inspection found and
repaired two owned compile defects exposed in the same log: the native taken-ID
removal helper had been deleted during the runtime-ID counter cutover, and the
interpolation regression called a nonexistent context convenience method
instead of the Level's public interpolation snapshot. Exact-manifest managed
validation remains required.

## 2026-08-30 Managed Validation Handoff

The exact Runtime22 source set is attributed to coordinator session
`root-runtime22-checkpoint-atomicity-20260829`. Static acceptance currently
includes Rust 1.94.1 rustfmt for all owned Rust files, scoped `git diff --check`,
and focused structure guards for indexed take/restore, allocation-free runtime
reference traversal, runtime callback panic restoration, and paused
single-step debt behavior.

Two immutable validation-copy requests were accepted together without waiting
for the occupied Cargo lane. Functional copy job
`e7144133050547cd958b6aee6a7e186f` binds the full release `core-min` lib
regression; performance copy job `556e7112dc9b447e8fc1e39c573adaf8`
binds all `runtime22_performance_` gates. Both copies carry the same 30-path
Runtime22 overlay and pin sibling `zr_vm` commit
`b730b40d6f85fcf5d28e590681213cc512704524`. The requests were queued for
materialization, but neither copy survived artifact governance and neither
Cargo run was admitted. This snapshot carries no functional pass count or
timing result and must not be used as performance acceptance.

Current incremental source attribution for the overlay includes
`scene_system_registry.rs=30996055bcb3e7502d8b08f0504ed3f01da56202f7dcaba197ece65c31b6ddf4`,
`scene_system_registry/runtime_system_slots.rs=eff4059c03d557cf591713b980523580365c901511c8ccf66dbf62820511905b`,
`schedule_stage_plan.rs=e393d65e24bfbd254630d7beb7d7a1c7ced6cedbf0340069668c814fddb931ff`,
and
`fixed_update.rs=03d4eeb1f4365570999baa894420091421f2daa7e208b4095c549d16c7025c3b`.

The latest paired materialization attempts (`4bc65ff5f2d2408fa4f09ef3b97e63e9`
and `bb72404894cd4fe28f53931dc0958521`) reached the coordinator's
`artifact_governance` stage and were removed before Cargo admission with
`unmanaged_artifacts_detected`. The reported foreign path was
`D:\\ZirconBuilds\\mvp-test-fixtures-35880`; it is outside the Runtime22
overlay and was neither deleted nor absorbed. Consequently this attempt
supplies no functional pass count, timing row, or performance claim; an
exact-manifest managed copy remains required. The earlier pair
`e7144133050547cd958b6aee6a7e186f` / `556e7112dc9b447e8fc1e39c573adaf8`
failed at the same pre-Cargo stage with the additional foreign path
`E:\\ZirconBuilds\\editor08-validation-20260829`.

## Boundaries

This slice does not alter tooling, native-system storage, schedule topology, or
the fixed-step clock transaction. It does not claim state/effect/RNG rollback,
allocation-hook, lock-contention, RSS, Tracy, WPR, or energy evidence.
