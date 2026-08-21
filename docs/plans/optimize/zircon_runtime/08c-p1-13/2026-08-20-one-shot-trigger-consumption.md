# Runtime08C One-Shot State-Machine Trigger Record

- Date: 2026-08-20
- Owner: `optimize-runtime08c-one-shot-trigger-r1-01a00797-20260820`
- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-13
- Execution plan: `docs/plans/optimize/zircon_runtime/08c-p1-13-one-shot-trigger.md`
- Status: implementation and regression definition complete; managed validation pending

## Problem

`AnimationParameterValue::Trigger` remained in the player parameter map after
matching a transition. Any later evaluation from a compatible source state
could therefore match the same input again. Consumption also could not be
performed during condition evaluation because exit-time, interruption-source,
and clip-event admission may still reject or defer the transition.

## Change

- Compiled transitions now retain the unique Trigger parameter names from
  their authoring conditions. Evaluation exposes only the names belonging to
  the first condition-matching transition, preserving existing ordered
  arbitration.
- Ordinary transitions attach those names to the pending commit only after
  their exit-time gate opens. Active-transition arbitration copies an `Arc`
  handle for the exact selected candidate; its Trigger names are attached to
  commit only after source and transition pose sampling both succeed. No
  Trigger string is cloned on the frame path.
- The pipeline carries active state and consumed Trigger names in one update.
  The existing deferred-entity filter removes that whole update before commit,
  preserving the Trigger for a later retry.
- Commit removes a parameter only if its current value is still `Trigger`.
  Unselected Triggers and same-name values overwritten with Bool, Scalar, or
  another non-Trigger value are retained.
- The public `AnimationManager::evaluate_state_machine` path remains a pure
  compatibility evaluator and does not mutate caller parameters.

## Regression Contract

- `one_shot_trigger_evaluation_reports_only_the_selected_transition_triggers`
  locks selected-versus-unselected Trigger metadata.
- `one_shot_trigger_commit_consumes_only_current_trigger_values` locks atomic
  active-state commit, unrelated Trigger retention, and non-Trigger overwrite
  preservation.
- `one_shot_trigger_waits_for_exit_gate_then_is_consumed_once` exercises the
  production tick: the Trigger survives the closed gate, is removed when the
  transition begins, and remains absent on the following frame.
- `one_shot_trigger_zero_duration_pose_failure_commits_nothing_until_retry`
  locks the shortest partial-commit counterexample: a missing target pose for a
  completed transition changes neither active state nor Trigger until retry.
- `one_shot_trigger_interruption_waits_for_source_pose_then_consumes` proves a
  failed source sample retains both the previous crossfade and Trigger, then
  consumes the Trigger only after the interruption pose succeeds.
- `one_shot_trigger_deferred_clip_event_admission_retries_before_consuming`
  fills the real bounded Level event queue, proves the deferred frame retains
  active state/transition/Trigger, then drains capacity and commits on retry.
- Exact-file Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Independent final static review: Critical/Important/Minor = `0/0/0`;
  ordinary failure, interruption retry, bounded-queue deferral, nested commit,
  and exact selected-edge metadata were all reviewed against production paths.
- Cargo regressions: pending the next managed multi-task Runtime08C batch; no
  direct or competing Cargo process was started.

## Performance Contract

Trigger-name discovery and storage occur only when a state-machine asset
revision is compiled. Frames waiting on an exit gate copy only an `Arc` handle
to the exact selected transition metadata; they do not clone Trigger strings
or allocate a list. Commit reads the shared slice only when an ordinary or
interruption transition actually begins. This correctness slice adds no
standalone latency threshold. Its behavior group will share the Runtime08C
batch whose seven existing release gates emit 21-pair alternating-order
P50/P95 evidence.

## Remaining Plan Work

The runtime state is still distributed across the scene component,
`LevelSystem` playback maps, and pipeline caches. A single generation-owned
`AnimationMachineInstance`, parameter page, and rollback journal remain under
Runtime08C P1-13 and are not claimed by this slice.
