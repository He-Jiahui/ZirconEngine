---
related_code:
  - zircon_runtime/src/core/framework/input/input_action_map.rs
  - zircon_runtime/src/core/framework/input/input_action_state.rs
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/runtime/action_evaluator/consumed_input_index.rs
  - zircon_runtime/src/input/runtime/action_evaluator/generation.rs
  - zircon_runtime/src/input/runtime/action_evaluator/workspace.rs
  - zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - zircon_runtime/src/input/tests/action_mapping.rs
plan_sources:
  - docs/plans/mvp/03-f2-scene-runtime.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/performance/01/2026-08-14-runtime-input-ingress-current-review.md
reference_sources:
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputMappingContext.h
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/godot/core/input/input_map.h
  - dev/godot/core/input/input.cpp
doc_type: implementation-design
status: implementation_complete_static_reviewed_runtime_validation_pending
---

# Input Action Evaluation Generation And Workspace

## Decision

`zircon_runtime::input` remains the owner of raw input reduction and action evaluation.
`zircon_runtime::core::framework::input` keeps the persisted, serde-visible
`InputActionMap`, `InputAction`, `InputActionContext`, `InputBinding`, and
`InputActionState` contracts. The App host continues to own producer-side
coalescing, and Runtime09 continues to own UI route/capture/focus decisions.

The change is an internal evaluator compilation step, not a new input-routing
policy. `set_action_map` compiles the persisted strings and binding vectors into
one immutable generation. Each evaluation reuses a private evaluator
workspace. `DefaultInputActionManager` serializes its evaluator calls through
the existing mutex; directly constructed evaluators retain their public
`Send + Sync` boundary through a private workspace mutex. The manager calls an
exclusive internal path while holding its existing evaluator mutex, so normal
manager evaluation does not take a second lock. The change must not add a
global lock or bypass the manager facade.

## Current Cost Model

The pre-change evaluator already avoided the old full action-by-binding scan by
building a map-change-time binding lookup. It still created temporary
`BTreeMap`/`BTreeSet` collections on every evaluation:

- frame-axis construction allocated two ordered maps for current axes and
  transitions.
- consumed button, consumed axis, and active-context slices become new ordered
  sets even when the slices are empty.
- every action performs string-keyed index/context lookup, then output sets and
  maps clone public action strings.

For `A` actions, `B` bindings, `G` frame axes, `T` axis transitions, and
non-empty input filters `F`, the old steady evaluation has approximately
`O(A log A + A log C + B + (G + T) log(G + T) + F log F)` lookup work plus
transient ordered-collection allocation. A direct caller-slice scan is not an
acceptable replacement: it would make UI-consumed membership `O(B * F)`.
The new reusable index keeps `O(F log F + B log F)` membership while retaining
only integer indices into the caller slices, so it does not clone string-backed
`InputButton` values. The public `InputActionState` output still necessarily
owns action identifiers under its current API; that output materialization is
not an intermediate-allocation excuse to retain the other per-frame containers.

The 2026-08-14 ingress review proves that input is already lock and ABI heavy
before this layer. This slice only removes evaluator-local repeated work; it
does not claim to fix App producer batching, Runtime UI routing, or WPR power
data.

## Reference Evidence

Unreal Enhanced Input keeps mapping contexts as registered data and evaluates
actions from the applied context set in `EnhancedPlayerInput.cpp`; it separates
mapping/trigger processing from input-consumption policy. Its context API also
has distinct priority, filtering, and registration concepts. Zircon therefore
must not reinterpret `InputActionContext.priority` as a local "eat every lower
binding" flag: Runtime12's existing contract requires UI to provide the
consumed/unhandled input boundary before gameplay evaluation.

Bevy's `ButtonInput` preserves pressed and edge state separately and clears
edge state at a frame boundary. Zircon already follows that order:
`RuntimeDynamicSession::tick_frame` runs the level, then invokes
`InputManager::begin_frame`. The evaluator must remain a pure read of that
single frame state.

Godot keeps a persistent action map and updates cached action state from input
events. It supports the conclusion that map compilation belongs at map-change
time, while its global singleton/event-delivery model is deliberately not a
Zircon model to copy.

## Target Shape

`input/runtime/action_evaluator/` gains private implementation details only:

1. `ActionEvaluationGeneration` owns dense action slots in declared action
   order, contiguous binding ranges, context slots, and the construction-time
   string-to-slot maps required to translate caller-facing active contexts.
2. `ActionEvaluationWorkspace` owns reusable axis lookup vectors, active-context
   bits, and sorted indices into caller-owned consumed-input slices. Axis vectors use
   in-place ordering by `(physical axis, source order)` so duplicate samples retain the
   former last-write-wins semantics without stable-sort auxiliary storage. Consumed-input
   indices restore logarithmic membership without cloning string-backed buttons.
3. `InputActionEvaluator` owns the persisted `InputActionMap`, its immutable
   generation, and the workspace. `set_action_map` swaps all three atomically
   under the existing manager lock and resets workspace contents. Direct public
   evaluation locks its local scratch; the manager uses its existing exclusive
   lock to avoid nested hot-path locking.
4. Final projection alone materializes the existing owned
   `InputActionState`. Public action IDs, ordering, serde shape, chord/axis
   semantics, disabled contexts, and UI-provided consumed input remain
   unchanged.

The first implementation must use `Vec` ranges and slot-indexed boolean/value
arrays for per-action state. It may retain `BTreeMap` for construction-time
deterministic string lookup and for public output ordering. It must not expose
dense IDs in persisted project data or dynamic ABI until a separate contract
decision proves that migration safe.

## Semantics And Invariants

- An empty active-context slice means all enabled contexts, as today.
- A non-empty active-context slice enables only matching, enabled contexts;
  contextless actions stay active.
- `consumed_buttons` and `consumed_axes` are supplied by the upstream UI route.
  They gate matching bindings exactly as today; this evaluator does not own UI
  capture, popup, focus, or dispatch policy.
- `InputActionContext.priority` remains data for context registration and
  future arbitration. This slice must not silently make it behavior-changing.
- A binding's chord, axis direction, dominant absolute action value, activation
  and deactivation semantics remain byte-for-byte equivalent at the public
  state API.
- Rebinding creates a new generation before the next evaluation. No stale
  range, active bit, axis index, or action string can survive `set_action_map`.
- The evaluator stays one manager-local synchronization point. Evaluation does
  not introduce a second mutex, global cache, or per-action allocation path.

## Measurement Plan

Test-only evaluator counters now record generation builds, binding visits, frame-axis
source visits, consumed-input source visits, workspace storage growth, and distinct
public action records.
The focused scale fixtures cover 10/100/1,000/10,000 bindings and 10,000-item
consumed-button and consumed-axis sets without using the incremental action-map
builder, so builder de-duplication or direct-slice scan cost is not mistaken for
evaluator work.
Managed validation must capture baseline and after values for 1/4/8 gamepads,
empty and non-empty UI filters, at 30/60/120 Hz, and verify the same public
action state for chords, consumed inputs, disabled contexts, rebinding, axis sign
changes, and press/release edges.

Only after the managed Windows path is released may the product validation
record allocation bytes, p50/p95/p99 evaluator time, manager-lock wait, frame
CPU, and WPR/ETW power data. Artifacts must be rooted under approved `D:`,
`E:`, or `F:` locations; no historical C: captures are admissible.

## Implementation Order

1. Add a failing regression proving that repeated evaluations with unchanged
   map/frame do not grow evaluator workspace storage after warm-up, while
   public state remains equal.
2. Compile the immutable action generation on construction and map replacement.
3. Convert axis/context scratch to reusable workspace and index consumed input
   by caller-slice position, avoiding both per-frame value clones and `O(B * F)` scans.
4. Preserve existing action-state projection; test-only counters and
   10/100/1,000/10,000 scale fixtures are now present for the later managed
   measurement stage.
5. Perform static second review for stale string lookup in the hot loop,
   changed UI filtering semantics, workspace reset correctness, and mutex
   layering. Managed Cargo and Windows profiling remain a later validation
   stage, not evidence fabricated by this source slice.
