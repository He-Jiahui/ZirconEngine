---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: scene-animation-optional-feature-compile-boundary
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/animation/clip_event_sampling.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_runtime/src/scene/level_system/frame_state.rs
  - zircon_runtime/src/scene/ecs/events/store.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/diagnostics.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/ik_postprocess.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/core/framework/animation/ik_command_error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/scene/tests/render_extract/level_source_guards.rs
  - zircon_runtime/src/scene/tests/level_system_frame_state.rs
tests:
  - python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary -v
  - cargo +1.94.1 test -p zircon_runtime --lib level_system_constructs_and_replaces_world_without_animation --no-default-features --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib transactional_world_replacement_discards_retired_animation_events --features animation --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib transactional_world_replacement_clears_retained_pose_resources_without_animation_runtime --no-default-features --features physics-contracts --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib transactional_world_replacement_preserves_staged_lifecycle_callback_events --no-default-features --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --lib replacement_epoch_and_empty_mode_prepare_without_duplicate_resets --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --lib replacement_epoch_retires_pending_diagnostics_from_all_evaluators --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --lib replacement_epoch_retires_deferred_ik_commands_and_rejects_late_old_epoch --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib replacement_epoch_retires_deferred_ik_commands_and_rejects_late_old_epoch --features animation --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --test animation_ik_contract --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --test runtime_physics_animation_tick_contract queued_two_bone_ik_runs_after_base_animation_pose_and_before_publication --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_animation_runtime --lib clip_player_time_is_deferred_until_event_batch_admission --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_sdk --lib runtime_registration_builder_hides_module_owner_sequence --locked --jobs 1 -- --nocapture --test-threads=1
  - Runtime animation-enabled LevelSystem clip-event behavior remains covered without a scene-to-animation reverse dependency
---

# Frameworks01: scene animation optional-feature compile boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：per-World scene-system callback factory SDK forwarding validation
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：Frameworks01 owns the declared `scene->animation` reverse dependency and the optional-domain decomposition boundary. Plugin SDK must not make an optional Runtime animation domain mandatory merely to compile its runtime registration builder.

## 失败现象与复现证据

Managed source-bound job `d30d4e49a2aa474f9143efbf547fbff6` /
`5dffe135b5f54b9aa1f3da35ef9a60c0` ran:

```text
cargo +1.94.1 test -p zircon_plugin_sdk --lib runtime_registration_builder_hides_module_owner_sequence --locked --jobs 1 -- --nocapture --test-threads=1
```

It terminated `exit 101` before the requested SDK test binary ran. The SDK's `runtime` feature selects
`zircon_runtime` without its default optional domains, and that dependency compile produced three errors:

- E0432 at `zircon_runtime/src/scene/level_system.rs:6`: unconditional `crate::animation` import while `lib.rs` gates `animation` behind `feature = "animation"`.
- E0432 at `zircon_runtime/src/scene/level_system/frame_state.rs:4`: the same unconditional dependency on the animation clip-event cursor.
- E0689 at `zircon_runtime/src/scene/level_system.rs:314`: `emitted_event_bytes = 0` has no concrete numeric type for `saturating_add(batch.emitted_event_bytes)`.

The lower Runtime callback GREEN job had already executed all four callback tests successfully, so this
record is compile-blocker evidence only and does not reinterpret either callback or SDK behavior.

Managed no-default-features retry `d2bad3c6a3dc40d5860f11d1400003e9` /
`4b6f8c4dc4e542fdac9c5e16fb4aa62e` reached the `zircon_runtime` lib-test compiler but exited
`101` before its requested test executed. It proved two further local boundary leaks: a statement-level
`#[cfg]` at `level_system.rs:181` (E0658) and four animation-pose tests in
`scene/tests/level_system_frame_state.rs` that compile without the `animation` feature (E0599).
It also exposed an independent Frameworks01 M1 test-owner error: `core/runtime/tests/tasks.rs` imports
the now-private `core::framework::render::environment` module (E0603). That file belongs to the active
M1 contracts/kernel test-boundary session and is not repaired by this failure scope.

## 最低共享层根因

`scene::level_system` stores clip-event sampling cursor/state in its always-compiled frame-state owner
and imports `crate::animation` unconditionally. `zircon_runtime::animation` is optional, so a consumer
that legitimately omits the animation domain compiles an invalid scene module. The untyped event-byte
accumulator is a second latent compile error in the same optional code path. The mixed render-extract
source-guard child also compiled three animation-only assertions under every feature selection. Only
those animation-specific cases may carry the feature boundary; its two always-on scene/render
architecture guards must remain compiled without the animation feature.

## 架构修复验收

- Scene's always-compiled LevelSystem and frame-state owners build when Runtime animation is absent.
- Clip-event sampler/cursor state has an explicit animation-enabled owner or scene-facing contract boundary; no direct always-on scene import reaches the optional animation module.
- The event-byte budget preserves its concrete byte-count type and saturating semantics.
- The exact Plugin SDK command above compiles and executes exactly its requested test.
- Animation-enabled clip-event behavior and the original Plugins01 four-test callback GREEN remain passing under managed source-bound validation.

## 禁止临时方案

- Do not force `animation` into the Plugin SDK runtime feature or otherwise broaden consumer feature selection to hide the broken boundary.
- Do not restore a scene-to-animation facade, re-export, alias, or a test-only cfg bypass.
- Do not weaken clip-event budgeting, event continuity, or the SDK test filter.

## 修复结果与回传

2026-08-01 current-source hard cut:

- `level_system.rs` and its frame-state owner now gate every direct optional animation import,
  state field, clip-event cursor, and animation-only method behind `feature = "animation"`; the
  always-compiled LevelSystem/frame snapshot remains available without that domain.
- The final reset call in `replace_world_and_reset_runtime_state` is enclosed by an item-stable cfg
  block instead of a statement-level cfg attribute. This closes the remaining Rust E0658 boundary
  without enabling animation for Plugin SDK consumers.
- The clip-event byte accumulator is explicitly `usize` and retains saturating addition. Animation
  pose/source-guard tests carry the animation cfg, while the two scene/render architecture guards
  remain always compiled.
- The first independent review found that a module-level cfg still hid all five mixed
  `level_source_guards` cases. The cfg now sits only on the three animation behavior tests and their
  parent imports/helper; the snapshot-adapter and inactive-camera architecture guards compile in
  every feature selection. A focused source guard fixes this at `3` animation-only plus `2`
  always-on cases.
- Scoped Rust 1.94.1 rustfmt, diff-check, production feature-boundary source guards, and test cfg
  guards are GREEN. The first independent review was C0/I1/M0 and its Important finding is repaired;
  the fresh exact-scope re-review is C0/I0/M0 Ready. Managed Runtime no-default, Plugin SDK, and
  animation-enabled gates are still required before the failure can return as fixed.

2026-08-11 replacement-epoch and bounded-queue forward fix:

- The optional compile fix no longer stops at `#[cfg(feature = "animation")]`. Clip-event cursor,
  request, limits, batch, event DTO, and sampler interface now have one neutral owner at
  `core::framework::animation::clip_event_sampling`; the concrete project-asset sampler remains in
  the optional `animation` implementation.
- `LevelSystem` retains only bounded queue/retry/age/overflow policy and consumes the neutral sampler
  contract. The animation plugin constructs `ProjectAnimationClipEventSampler` and imports the event
  DTO from the new contract owner. The former `zircon_runtime::animation::AnimationClipEvent` export
  is deleted rather than retained as a compatibility alias.
- TDD reproduced the two production `scene→animation` references before the hard cut. The new
  production-domain regression is GREEN with zero reverse references; Python compile, exact
  rustfmt, stale-path scan, and diff-check are GREEN. Managed Runtime no-default, Runtime animation,
  and animation-plugin gates remain required before fixed return.
- Independent review rejected the first generation seal because ordinary animation work advances
  `World::world_generation`; the producer could reject its own pose after updating player time or a
  bone transform, while old-World events, playback state, and pipeline writes could still cross a
  concurrent replacement. The same review found a 32-item drain budget but no queue storage bound,
  and unattempted tail entries did not age. The replacement-specific epoch is now separate from
  ordinary component mutation. Direct and transactional replacement retire it before resetting all
  World-coupled state under the World lane.
- Every plugin World mutation, event publish, queue operation, pose publication, and playback
  publication now rejects a retired epoch. Pose frames are stamped with the current ordinary World
  generation under the World lock, so valid component writes no longer suppress publication.
  Transactional replacement clears every inherited retired-World event queue before staged
  lifecycle callbacks commit, preserving registered event types/readers/observers and events newly
  emitted by those callbacks. Retained skeletal-target/simulated-pose resources are cleared even
  when the optional animation runtime is absent.
  The plugin pipeline resets entity-bound projection, presentation, transition, nested-machine state,
  and evaluator diagnostic queues/deduplication on replacement-epoch changes and active-to-empty
  transitions without repeating empty-state
  resets or resetting again on empty-to-active transition in the same epoch. Reusable compiled-asset
  caches remain retained. Animation runtime behavior moved from `level_system.rs` to
  `level_system/animation_runtime.rs`.
- The private queue limit is `32 * 8 = 256` ranges. Admission returns one explicit result for every
  entity-owned batch: `Admitted`, retryable `Deferred`, or terminal `RejectedOversized`. Ranges for
  one entity are never split. A batch larger than the absolute queue capacity emits
  `animation.clip_event_batch_capacity_exceeded` and does not block later owners; ordinary capacity
  pressure defers the remaining owner batches without advancing their state. The plugin rotates from
  the last admitted owner. Retryable deferred owners commit none of their clip/sequence player state,
  sequence properties, poses, evaluator diagnostics, IK commands, or projection revision deltas.
  Terminal oversized rejection emits its explicit diagnostic and advances non-event state so the same
  impossible event batch is not retried indefinitely.
  Targeted state-machine rollback intersects deferred owners with the checkpointed entity set, so an
  unrelated paused machine is not erased. No full projection-map checkpoint is cloned on the tick hot
  path. Partial/unavailable queued work rotates to the tail, and pending age is measured in drain
  windows rather than wall frames. Queue memory is O(256); each drain ages at most 256 entries and
  invokes at most 32 samplers. This is a correctness and ownership repair, not a performance claim.
- Reference review used Unreal's per-evaluation `AnimNotifyQueue` reset, Bevy's World identity kept
  separate from change ticks, and Fyrox's fixed animation event queue. Zircon deliberately retains
  bounded cross-frame cursor retry, but assigns it an epoch-tagged owner and explicit diagnostics.
  Evaluator diagnostics are retired with their World epoch. IK queue and drain operations carry the
  replacement epoch explicitly; each manager keeps a monotonic per-World epoch high-water mark, clears
  retired commands when the epoch advances, and rejects late old-epoch producers instead of executing
  them against the replacement World.
- Focused regressions now cover ordinary mutation, direct and transactional replacement, stale
  World/state writes, retired-event isolation, staged lifecycle callback event survival,
  physics-contract-only retained-pose cleanup, pipeline epoch/empty-mode preparation, segmented
  overflow progress, deferred clip-player/state-machine commit, retired diagnostic and IK isolation,
  drain-window age/fairness, multiline/grouped/alias exports, and all five plugin pipeline World-write
  owners. The expanded Python boundary suite is
  7/7 GREEN; canonical managed Runtime/plugin gates and a fresh immutable second review are pending.

2026-08-13 second-review cache-retention forward fix:

- The immutable exact35 review was stable and returned C0/I0/M1. Its Minor found that resetting
  replacement-bound evaluation state cleared every worker evaluator and the sequence cache, which
  contradicted the stated reusable compiled-asset cache policy and caused needless recompilation on
  same-epoch active-to-empty transitions.
- Reset now clears diagnostic pending/dedupe/order state in place for the primary and every worker
  evaluator, retaining their revision-aware skeleton/clip caches. World-bound compiled sequences are
  retired only when the replacement epoch changes; an active-to-empty reset in the same epoch keeps
  them and still clears entity-bound projection, presentation, transition, and diagnostic state.
- The boundary suite now rejects worker-vector clearing and requires sequence-cache retirement to
  remain behind the explicit World-bound-cache branch. Python boundary tests are 7/7 GREEN; Rust
  1.94.1 scoped rustfmt and diff-check are GREEN. A fresh immutable second review and all canonical
  managed Cargo gates remain pending.
- The fresh immutable review then detected concurrent `scene/mod.rs` drift from the independent
  Plugins01 scene-hook hard cut and Runtime08 detached-batch transaction. The clip-event admission
  DTOs were therefore moved from the private LevelSystem module into the existing neutral
  `core::framework::animation::clip_event_sampling` owner. Runtime tests and the animation plugin
  import that contract directly, and the scene-root re-export is deleted rather than retained as a
  compatibility facade. The failure closeout no longer claims the shared scene root blob.

2026-08-13 post-receipt contract-layer correction:

- Snapshot 1659 exact34 had a stable C0/I0/M0 review, static boundary 7/7, and twelve durable managed
  validation receipts against source manifest
  `fa49e73ef8c8c70fb89f3699a5c72b244e26bf883296fed493952400695ee403`. A later M1 physical-crate
  dependency audit found that the newly added `AnimationManager::drain_ik_commands_excluding`
  signature named `crate::scene::EntityId`. Those receipts remain predecessor execution evidence,
  but they cannot support acceptance of the corrected source.
- The canonical identity already exists at `core::framework::scene::EntityId`; the manager contract
  now imports that neutral type beside `WorldHandle`. No runtime representation, ABI width, or
  behavior changed, and no scene facade alias or compatibility export was added.
- A focused source guard first reproduced the reverse dependency as RED. The first immutable exact3
  review returned C0/I1/M0 because an exact-string assertion could be bypassed by an aliased,
  grouped, or whitespace-split `crate::scene` import. The guard now reuses the repository Rust
  lexical code view and `use`-tree parser, rejects every direct scene-implementation path, and has
  mutation regressions for aliases, grouped/multiline imports, raw identifiers, fully qualified
  paths, and comment/string false positives. It still locks the neutral import and direct
  `[EntityId]` signature. The expanded Frameworks01 boundary suite is 9/9 GREEN; Rust 1.94.1 scoped
  rustfmt, diff-check, and the core/framework stale-path scan are GREEN with zero old-path hits.
  The fresh exact3 immutable review kept pre/post fingerprint
  `f0138f1f56061ed5aa425b1fea60ab088a45a870e5df36ec9f552563a06d15c6`, drift 0, and returned
  C0/I0/M0. New managed receipts are still required before fixed return.

Open state: `neutral_entity_identity_successor_validation_pending`;
no SDK, Runtime, or plugin Cargo pass is claimed.

## 2026-08-22 current-source borrowed-set guard convergence

- Current HEAD `bee4c707b714738346b49bba15c59468b8bd9b39` retains the neutral
  `core::framework::scene::EntityId` owner and zero `scene -> animation` production edges. The
  Frameworks01 Python boundary suite initially returned 8/9 because its source-shape assertion still
  required the predecessor `&[EntityId]` parameter after Runtime08C hard-cut the contract to a
  borrowed `&BTreeSet<EntityId>`.
- The Runtime08C source review shows the change removes one intermediate `Vec` allocation and replaces
  up to 4,096-by-2,048 linear membership comparisons with ordered-set lookup while lending the
  already-owned deferred set directly to both animation managers. Frameworks01 therefore keeps the
  production implementation unchanged and advances only the stale guard: it now requires the neutral
  identity import, the borrowed ordered-set signature, and the existing lexical rejection of every
  `crate::scene` implementation-path variant.
- Fresh command
  `python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary -v` is GREEN 9/9 in
  41.762 seconds. Managed no-default Runtime, animation-enabled Runtime, Plugin SDK, animation plugin,
  independent review, fixed return, and coordinator commit remain pending; this Failure stays open.
- Runtime08C's pending ignored release gate times only legacy linear membership versus borrowed
  ordered-set membership. It does not include the manager lock, `mem::take`, two-output `partition`,
  queue replacement, frame-level attribution, or power sampling. That gate can validate the lookup
  algorithm's expected scale change, but it cannot by itself prove end-to-end IK-drain latency or
  engine power convergence; those claims remain with the Runtime08C profiling owner and are not
  claimed by Frameworks01.
- The local Unreal reference re-check confirms `FAnimNotifyQueue` owns its per-tick notify arrays,
  `FAnimInstanceProxy::PreUpdate` resets that queue every update, and queue append preserves notify
  state uniqueness. This supports Zircon's evaluation-owned reset/merge boundary. Unreal does not
  prescribe Zircon's deferred-entity container, so it is not evidence for the `BTreeSet` performance
  ratio; that choice remains subject to Zircon's own measured gate.
- The full current-source drain review also finds the fallback and first-party plugin managers still
  duplicate the same per-world lock, replacement-epoch, capacity and queue-partition algorithm. Each
  drain performs `mem::take` plus a two-output `partition` while holding the manager lock, and the
  diagnostic queue repeats the same retained/admitted pattern. Runtime08C's current P1-17 plan is
  explicitly scoped to borrowed-set membership and intermediate entity-vector removal, so its
  microbenchmark cannot accept the duplicate queue owner, lock duration or two-Vec allocation cost.
  Those structural costs require their own Runtime08C profiling/owner-convergence slice before any
  claim that the complete IK admission algorithm is optimal; Frameworks01 does not rewrite that
  foreign implementation under this optional-feature boundary Failure.

## 2026-08-23 r9 current-HEAD confirmation

- On current HEAD `f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`,
  `core/framework/animation/manager.rs` still imports the neutral `EntityId`/`WorldHandle` contract and
  accepts `deferred_entities: &BTreeSet<EntityId>` without a `scene -> animation` implementation edge.
- Fresh command
  `python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary -v` remains GREEN
  9/9 in 32.794 seconds with `TEMP`/`TMP` routed to the repository E-drive coordinator state directory.
  This refresh confirms only the static optional-feature boundary. Managed Rust product gates,
  end-to-end IK-drain profiling, independent exact4 review, Failure return, and coordinator commit
  remain pending, so the Failure stays `open`.

## 2026-08-23 r9 structural algorithm reassessment

### Current-source findings

- The borrowed-set change is locally sound but is not an end-to-end engine optimization result.
  Both `zircon_runtime/src/animation/manager/mod.rs` and
  `zircon_plugins/animation/runtime/src/manager.rs` still own an independent
  `Mutex<HashMap<WorldHandle, WorldIkCommandQueue>>`, the same 4,096-command limit, the same
  replacement-epoch state machine, and the same lock-held `mem::take(...).partition(...)` drain.
  Every World therefore contends on one manager-wide mutex, and each non-empty drain materializes
  separate retained/admitted vectors while the lock remains held.
- Current tracked production source has one consumer of `drain_ik_commands_excluding` in the plugin
  tick, but no production caller of `queue_ik_command`; all enqueue call sites are tests. The current
  21-pair ignored benchmark measures only legacy linear membership versus borrowed ordered-set
  membership over 4,096 commands and 2,048 deferred identities. It does not execute map lookup,
  mutex acquisition/hold, epoch handling, command movement, two-vector partition, queue writeback,
  World mutation, IK application, allocation tracing, frame time, or energy capture. Its relative
  result must not be promoted to a product bottleneck or power-convergence claim.
- The duplication is also an ownership defect: Runtime and the first-party animation plugin each
  publish a distinct `DefaultAnimationManager` and an `animation.runtime` module descriptor with the
  same qualified service names. Tests explicitly assert that the two manager types differ. This is
  incompatible with the hard-cut architecture goal of one implementation owner behind the neutral
  `AnimationManager` contract.
- Unreal keeps `FAnimNotifyQueue` on each `FAnimInstanceProxy`, resets it during `PreUpdate`, retains
  its `TArray` storage through `Reset`, and merges instance queues while preserving state-notify
  uniqueness. That evidence supports instance/World-local queue ownership and reusable storage; it
  does not validate Zircon's global manager mutex or provide an IK-specific latency number.

### Required structural direction before optimization

1. Runtime08C/Plugins04 must choose one implementation owner. The recommended hard cut is to keep
   the neutral trait and value contracts in Runtime, keep the first-party implementation/module in
   `zircon_plugin_animation_runtime`, and delete the duplicate Runtime manager/module registration
   instead of retaining a fallback, alias, re-export, or second service identity.
2. Establish a real product producer/consumer trace before changing the queue. If product enqueue
   count remains zero, the queue is dead capability debt rather than an MVP hot path and should not
   consume optimization budget.
3. If the product trace proves demand, replace the manager-wide map lock with a bounded per-World
   inbox owned by the animation evaluation lifecycle. Keep epoch rejection and capacity admission at
   the inbox boundary; drain must reuse retained storage and avoid rebuilding two vectors under a
   cross-World lock. The concrete container is selected only after measured contention and deferred
   ratios are known.
4. Profile the full path before and after with Windows ETW/WPA plus engine spans: enqueue count and
   latency, drain lock wait/hold P50/P95/P99, allocations/bytes after warmup, commands drained and
   retained, World count, deferred ratio, epoch resets, animation-tick CPU, frame time, and process
   energy. Required scenarios are 1/4/16 Worlds, 0/256/4,096 commands, 0/50/100% deferred entities,
   epoch rollover, and concurrent producers. Compare identical product captures with Unreal Insights
   queue/update markers; do not claim engine parity from the membership microbenchmark.

This reassessment is planning evidence only. The animation implementation paths belong to their
Runtime08C/Plugins04 owners and are not modified by Frameworks01 r9. Managed Cargo, product profiling,
cross-owner hard-cut implementation, independent review, fixed return, and commit remain pending.

### World-lifecycle and IK-reference confirmation

- The lifecycle risk is now bounded precisely. `LevelSystem::replace_world_and_reset_runtime_state`
  preserves its `WorldHandle` and advances only `world_replacement_epoch`, so repeated replacement of
  one Level does not grow the IK map. New Levels are different: `DefaultLevelManager` allocates handles
  with a monotonic `AtomicU64`, retains every Level in its own `HashMap`, and exposes no remove/unload
  operation through either its lifecycle implementation or the neutral `LevelManager` trait. The
  animation drain calls `queues.entry(world).or_default()` even when no command was ever submitted.
  Consequently every newly created Level that reaches animation tick creates an empty animation-map
  slot retained for the manager lifetime. This is O(L) metadata for L ticked Level handles, independent
  of useful IK work, and there is no lifecycle callback through which the animation owner could evict it.
- With n queued commands and d deferred entities, the borrowed `BTreeSet` drain is analytically
  O(n log d), improving the predecessor O(n*d) membership scan, but all Worlds serialize through one
  mutex. `mem::take` transfers the old command vector into the iterator; `partition` builds new retained
  and admitted vectors, so the old backing allocation is not reused for the next retained queue. The
  retained capacity can remain as high as 4,096 commands per ticked Level, giving an upper bound of
  O(L * 4,096) queued command slots plus O(L) map state in each duplicated manager. These are asymptotic
  bounds from current source, not measured allocation or energy results.
- The plugin tick obtains the Level's World mutex through
  `with_world_mut_if_replacement_epoch` before it calls the manager drain. Map lookup, epoch handling,
  membership checks, partition allocation, and command movement therefore run while both the World lane
  and the manager-wide lane are held. This needlessly lengthens the World critical section and makes
  independent Worlds wait on shared manager state. No reverse lock order is currently proven because
  production has no enqueue caller, so this record does not claim a demonstrated deadlock.
- The IK-specific Unreal reference is stronger than the earlier notify-queue analogy.
  `FAnimNode_TwoBoneIK::EvaluateSkeletalControl_AnyThread` produces bone transforms directly during graph
  evaluation. `FAnimNode_SkeletalControlBase` owns its `TArray<FBoneTransform>` and calls
  `BoneTransforms.Reset(BoneTransforms.Num())` before evaluation, retaining node-instance storage rather
  than routing IK through a process-wide manager queue. This supports graph/evaluation-instance ownership
  for Zircon IK and reusable local output storage; it is structural reference evidence, not a latency or
  power-parity measurement.

The MVP direction is therefore tightened: because the current product trace has zero enqueue callers,
Runtime08C/Plugins04 should hard-cut the global `queue_ik_command`/drain capability and express TwoBoneIK,
LookAt, and future controls as animation-graph evaluation inputs/outputs owned by the active Level
evaluation. Only a demonstrated external producer contract may justify a replacement inbox; that inbox
must be Level-lifecycle-owned, bounded, removable with the Level, and profiled end to end before its
container is selected. The zero-command scenario must prove zero map insertion and zero per-frame queue
allocation after warmup. Nonzero scenarios retain the full ETW/WPA and engine-span matrix above.

## 2026-08-24 Plugins04 global IK queue hard-cut convergence

- Plugins04 session `plugins04-global-ik-queue-hard-cut-r1-20260824` implemented the structural
  direction recorded above: the neutral manager no longer exposes `queue_ik_command`,
  `drain_ik_commands`, or `drain_ik_commands_excluding`, and the production animation tick no longer
  drains a process-wide IK inbox. The pure TwoBone and LookAt solvers remain plugin-owned.
- The Plugins04 output record explicitly reopens M5-T1 product integration. The graph evaluator still
  lacks the compiled pose program, per-entity graph instance, reusable component/model-space workspace,
  and graph-local skeletal-control node required to apply those solvers in production. Frameworks01
  does not restore the retired queue or reinterpret solver-only tests as a product IK closure.
- The Frameworks01 static guard had become stale because it still required the predecessor
  `EntityId`/`BTreeSet` drain signature and a tick call to `drain_ik_commands_excluding`. It now locks
  the hard-cut invariant instead: neutral `WorldHandle` ownership remains below the optional domain,
  no scene implementation path leaks into the manager, and all three process-wide IK inbox symbols
  stay absent from the neutral manager and production tick.
- Fresh command `python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary -v`
  is GREEN 9/9 in 39.915 seconds (42.753 seconds process wall time). Python syntax compilation and
  scoped `git diff --check` are also GREEN on the exact current files.
- This update closes only the stale static assertion. The optional-feature failure remains `open`
  pending its managed Runtime no-default, animation-enabled Runtime, Plugin SDK, and animation-plugin
  product gates, independent exact-source review, and coordinator failure return. Plugins04 retains
  ownership of graph-local IK product integration and its required profiling; no latency, power, or
  engine-parity result is claimed here.

## 2026-08-28 parameter-owner hard-cut performance review

- Current-source review found that graph and state-machine scene schema plus ECS player components
  still stored `BTreeMap<String, AnimationParameterValue>`, while the evaluation projection copied
  each map into a per-entity `AnimationParameterSet` snapshot. Stable playing entities therefore
  performed a full map equality in both scan paths every frame before cloning the snapshot into the
  request. The two snapshot maps also repeated entity lookup, retention, and owner storage already
  provided by the ECS component.
- The existing production-shaped proxy recorded in Frameworks01 measured that stable equality path
  at 0.196/0.742/3.029/13.123 us median for 8/32/128/512 parameters. The earlier independent-revision
  path measured 0.028/0.089/0.414/1.815 us, while map clone plus dense-row rebuild measured
  2.742/16.957/144.017/491.103 us. These are standalone CPU/data-structure measurements, not frame,
  energy, or engine-parity evidence.
- The earlier revision-only implementation was correctly rejected because `active_state` and
  parameters shared one ECS component change tick. The current hard cut removes that invalidation
  coupling: both persisted scene owners and both ECS player owners now store
  `AnimationParameterSet`, whose process-local content revision changes only through its COW
  mutation API. Custom serde keeps the scene wire value as the parameter map and reconstructs a
  fresh runtime revision on load; no old map field, alias, or compatibility owner remains.
- The selected structural optimization is therefore to remove both projection snapshot maps and
  clone the component-owned `AnimationParameterSet` directly into graph/state-machine requests.
  Stable request preparation becomes one `Arc` clone plus the existing request push, O(1) in
  parameter count, while actual parameter mutation remains O(P) for COW/fingerprint refresh over P
  parameters. The guard must reject `graph_parameter_snapshots`,
  `state_machine_parameter_snapshots`, and `parameters.synchronize(&player.parameters)` after this
  hard cut. Managed product execution, frame attribution, allocation/power capture, fixed return,
  and acceptance remain pending.

Implementation followed that direction without touching the MVP00-owned `scene_asset.rs`. Both
scene asset parameter fields and both ECS player fields now use `AnimationParameterSet`; explicit
map literals migrate through `Into`, and `FromIterator` preserves collection construction without
an old map owner. The obsolete public `synchronize(&AnimationParameterMap)` entry and both plugin
snapshot maps are deleted. Graph/state requests now clone the component owner directly, preserving
its revision, fingerprint, and shared `Arc` values.

The focused TDD contracts first failed on both old schema/component fields, then on both old
snapshot scans. The final exact static batch
`python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary tools.tests.test_frameworks_03_server_feature_boundary -v`
is 31/31 GREEN in 45.112 seconds (49.885 seconds process wall time). Exact Rust 1.94.1 scoped
`rustfmt --check`, scoped `git diff --check`, and a current-file scan for old parameter-map owners,
snapshot maps, and player-map synchronization are GREEN with zero matches.

A temporary production-shaped Rust 1.94.1 release probe compared the predecessor per-entity
`BTreeMap` lookup plus full stable-content equality and request clone against the new direct owner
clone. Each process used 11-round medians; three independent process runs reported:

| parameters | predecessor median range | direct clone median range | speedup range |
| ---: | ---: | ---: | ---: |
| 8 | 142.776--168.143 ns | 14.139--17.661 ns | 8.92x--10.86x |
| 32 | 572.070--703.366 ns | 16.808--21.907 ns | 26.11x--39.64x |
| 128 | 2,280.830--5,597.150 ns | 14.658--27.043 ns | 127.06x--206.97x |
| 512 | 9,179.740--13,625.450 ns | 14.133--17.660 ns | 648.49x--804.68x |

The probe source SHA-256 was
`5030d7dcd568a10ca6f6f611e82b118c3419a2e023ff719f27efcc9176ee9f72`; the D-drive executable at
`D:/zircon-frameworks01-r12-animation-parameter-bench-20260828/animation_parameter_owner_bench.exe`
has SHA-256 `079d97c25c07b0869717cd599a6bfccb4d9e7b3121b5c9abbd416646f2dc8949`.
The direct path is flat within measurement noise across parameter count, so the predecessor O(P)
stable-compare bottleneck is absent from this exact data-structure proxy. This does not establish
product frame time, allocation after full plugin warmup, process energy, or Unreal parity. The
Failure remains `open` with state
`parameter_owner_and_projection_hard_cut_source_implemented / static_and_proxy_green /
managed_product_validation_pending`.
