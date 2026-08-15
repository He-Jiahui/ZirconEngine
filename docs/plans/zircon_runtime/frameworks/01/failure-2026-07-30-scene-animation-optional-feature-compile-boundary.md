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
