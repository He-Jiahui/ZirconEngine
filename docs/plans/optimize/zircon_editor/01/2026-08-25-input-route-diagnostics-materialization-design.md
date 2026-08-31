# Input route execution and diagnostics materialization design

Date: 2026-08-25

Status: current-source architecture review, implementation plan and preparatory ownership slices.
Pointer trace construction consumes its completed route and moves the existing bubble/root vectors;
normal pointer dispatch also borrows its original input/metadata until the single result move. The
conditional trace materialization and prepared-route transaction are not implemented, and no
product latency improvement is claimed. Managed Rust and Editor validation is still required.

## Decision

The input hot path must stop using the serialized diagnostic route trace as execution state.
Zircon needs two distinct products from one routing pass:

1. compact execution facts consumed immediately by hover, press and capture state; and
2. an optional, owned diagnostic snapshot materialized only under an explicit capture policy.

The routed bubble path already exists in `UiPointerRoute`. Its ownership should move once into the
active-pointer table. Normal pointer movement must not clone that path into diagnostics and then
clone it again out of diagnostics. Full trace capture remains available for tests, tooling and
support sessions, but it is not the product default.

## Preparatory ownership slice implemented

`zircon_runtime/src/ui/surface/input/pointer.rs` now passes the completed `UiPointerRoute` by value
to `annotate_pointer_route_trace`. `route_policy.rs` derives preview/focus/scalar facts first and
then moves `route.bubbled` and `route.root_targets` into the public trace. This removes two
unconditional vector clones without changing trace contents, route semantics or the public DTO.

A focused source contract was RED against the prior borrowed signature and both clone expressions,
then is GREEN 2/2 after the ownership transfer. Both production files pass scoped rustfmt and
diff-check. This is a preparatory constant-factor reduction only: preview/focus/route-step/popup
materialization and the manager's diagnostic-to-hover clone remain, so the terminal asymptotic
contract below is still open. No Cargo validation was run.

The second ownership slice removes the unconditional `UiInputEventMetadata` clone and complete
`UiPointerInputEvent` clone from `surface/input/pointer.rs`. The route call and optional text probe
borrow the original pointer; pointer ID, source and click count are copied as scalars; the original
pointer is then moved once into `UiInputDispatchResult`. Rich-link activation accepts only click
count and route, while pointer trace construction accepts only pointer source/ID and the owned
route. A `UiPointerEvent` scalar-field clone remains for the lower dispatcher.

This matters because `UiInputEventMetadata` can own window and surface ID strings. The old metadata
clone plus full-pointer clone could duplicate both strings twice on every normal pointer event even
when text/rich-link handling returned immediately. Text interactions may still clone their event
when they produce a separate merge result; that narrower path remains explicit. The ownership
source contract was RED for the two full clones and full-pointer helper signatures and is GREEN
3/3; the combined route/rich-link/input ownership set is GREEN 7/7.

## Current-source proof

The current route and manager pipeline performs diagnostic ownership work for every pointer event:

- `zircon_runtime/src/ui/surface/input/pointer.rs:20-126` constructs a routed pointer result, then
  unconditionally calls `annotate_pointer_route_trace` and `annotate_result_route_steps` before
  returning.
- `zircon_runtime/src/ui/surface/input/route_policy.rs:79-115` reverses the bubble path into a new
  preview vector, builds a focus path and deep-clones every popup ID into a new `Vec<String>`.
  The preparatory slice now moves the original bubble and root-target vectors rather than cloning
  them, but all other diagnostic materialization is still unconditional.
- `route_policy.rs:195-210` creates at least one owned source note string for every pointer event.
- `zircon_runtime/src/ui/surface/input/route_steps.rs:9-50` unconditionally derives another owned
  route-step vector from the diagnostic trace.
- `zircon_runtime/src/ui/dispatch/input_manager/manager.rs:435-478` then updates live pointer state
  from the diagnostic DTO. `active_pointer_hover_path` at lines 581-592 clones the diagnostic
  bubble path one more time.
- `zircon_runtime/src/dynamic_api/session/runtime_ui.rs:390-408` also reads capture state back from
  `diagnostics.route_trace` to maintain cross-surface capture ownership.
- `zircon_runtime_interface/src/ui/dispatch/input/result.rs:42-67` makes route trace, route steps and
  notes public serde DTOs with owned vectors and strings. An empty route currently cannot express
  whether capture was disabled or a captured trace was genuinely empty.

For a routed path of depth `H`, focus depth `F`, `R` root fallback targets and `P` open popups, the
current post-dispatch diagnostic work is `O(H + F + R + P)` time and memory even when nobody
observes diagnostics. After the ownership slice, a normal bubble route still allocates/copies
preview, focus, route-step and active-hover vectors, plus the notes vector/string and `P` popup
strings. The moved bubble/root vectors retain their existing allocation instead of copying it, but
diagnostic ownership still controls their destination and remains avoidable hot-path work.

The coupling is behavioral, not cosmetic. Clearing or conditionally skipping the current trace
without first moving hover/capture authority would make pointer hover and cross-surface capture
incorrect. Replacing the vectors with `Arc<Vec<_>>` would reduce some copies but still materialize
full diagnostics on every event and expose execution state through a serialization DTO. Both are
rejected.

### Primary-release hit duplication

There is a second authority problem before diagnostics are built:

- `zircon_runtime/src/ui/dispatch/input_manager/manager.rs:373-397` prepares double-click state
  before dispatch. `pointer_release_click_target` at lines 567-579 reads the pressed owner, calls
  `surface.hit_test` and searches the stacked hits.
- `zircon_runtime/src/ui/surface/surface/event_routing.rs:192-209` then routes the same event;
  `route_pointer_event_with_details` at lines 376-467 performs another hit query, derives the same
  release-inside-pressed fact and builds the authoritative route.
- When the hit-path target is selected, lines 389-395 clone `hit.path.bubble_route` even though the
  complete `UiHitPath` is then moved into the route at line 451. Public route state therefore owns
  two representations of the same bubble sequence before diagnostic projection.

The first hit cannot simply be deleted: click count is consumed by component/default handlers
during the second dispatch. The required repair is a two-phase synchronous route transaction:

```rust
struct UiPreparedPointerRoute {
    surface_frame_generation: u64,
    input_epoch: u64,
    route: UiPointerRoute,
    transition: UiPointerStateTransition,
}
```

`prepare_pointer_route` performs exactly one hit query and computes route/transition facts without
publishing focus, hover, press or capture mutation. The manager derives double-click count from the
prepared `click_target` and updates the event. `dispatch_prepared_pointer_route` verifies the frame
generation/input epoch, commits the prepared transition once, executes handlers with the enriched
event and returns execution facts. The synchronous manager call admits no unrelated mutation; a
generation mismatch is a typed fallback that prepares once more and is counted.

This transaction also owns one bubble sequence. Execution code borrows it, active-pointer state
receives it by move after dispatch, and full diagnostics clone it only when trace capture is
selected. Do not cache hit results globally by point: capture/focus/input policy and the published
frame generation are part of validity, and an implicit point cache would create stale-route bugs.

## Unreal reference contract

The checked-in Unreal Slate source separates route execution from optional debugging work:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:246-429`
  defines direct, leafmost, tunnel and bubble policies as cursors over a borrowed `FWidgetPath`.
- `SlateApplication.cpp:447-478` executes the chosen policy directly and stops when the reply is
  handled. The route path is the execution authority; it is not reconstructed from a diagnostic
  event record.
- The route debug scope at `SlateApplication.cpp:454-456` exists only under
  `WITH_SLATE_DEBUGGING`.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Debugging/SlateDebugging.h:15-17`
  compiles Slate debugging out of shipping/test builds, while
  `SlateDebugging.cpp:458-528` only constructs and broadcasts input event arguments when the
  corresponding delegate is bound. Routing observers are a separate optional list.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Trace/SlateTrace.h:14-18` gives Slate
  trace an independent compile-time gate; its bookmark macro at lines 92-96 also checks the
  runtime trace channel before emitting. Disabled builds replace trace macros with no-ops at lines
  127-139.

The transferable principle is that input semantics operate on a route path and reply, while
debugger/trace products observe that execution only when enabled. Zircon should retain its Rust
DTOs and richer deterministic tests, but their ownership must not define product pointer state.

## Required internal authority

Introduce an internal return envelope for surface-to-manager dispatch. Exact module names may
follow the runtime owner, but this ownership split is mandatory:

```rust
struct UiSurfaceInputDispatch {
    result: UiInputDispatchResult,
    execution: UiInputExecutionFacts,
}

#[derive(Default)]
struct UiInputExecutionFacts {
    pointer: Option<UiPointerExecutionFacts>,
}

struct UiPointerExecutionFacts {
    target: Option<UiNodeId>,
    capture_target: Option<UiNodeId>,
    hover_path: Vec<UiNodeId>,
    hit_path: UiHitPath,
}
```

`hover_path` is not a new copy: pointer dispatch transfers the existing routed bubble vector into
the envelope. The manager transfers it into `UiActivePointerTable::set_hovered_path`. Direct routes
may transfer a zero/one-node vector or use a small specialized representation if measurement
justifies it. `hit_path` retains the original `UiHitTestQuery` virtual-pointer information; popup
projection may change hit geometry but must not overwrite the physical query's
`UiVirtualPointerPosition`.

The existing public `UiInputDispatchResult` remains the semantic reply/host-output DTO. Internal
product code must not read `diagnostics.route_trace` to decide hover, capture, pressed targets,
cross-surface routing or any other behavior. Add a source contract that rejects new production
reads outside the diagnostic serializer/tooling boundary.

## Capture policy

`UiInputManager` (or its owner runtime session) must hold an explicit trace capture policy:

```rust
enum UiInputTraceCapturePolicy {
    Disabled,
    ErrorsOnly,
    Sampled { every: NonZeroU32 },
    Full,
}
```

- `Disabled`: retain scalar dispatch outcome fields, but do not allocate route trace vectors,
  route-step vectors, popup strings or informational note strings.
- `ErrorsOnly`: capture after execution only when dispatch is blocked, effects are rejected, a
  typed route fallback occurs or an invariant fails.
- `Sampled`: capture deterministically from input sequence so traces are reproducible.
- `Full`: preserve today's complete trace contract for focused tests, inspectors and support
  captures.

Add an explicit serialized capture-state field such as `route_trace_capture` with `Disabled`,
`NotSelected` and `Captured` values. Do not overload an empty `UiInputRouteTrace`, because an empty
root/popup path is valid data. Backward deserialization may default old payloads to `Captured` only
where the compatibility contract requires it; new product results must state their policy.

The product editor path uses `Disabled` by default. Test helpers that assert complete routes opt in
to `Full`; support tooling may select `ErrorsOnly` or `Sampled`. A global environment variable read
inside every event is forbidden. Configuration is resolved once into manager/session state.

## Publication sequence

One pointer event follows this order:

1. Prepare one hit lookup and route/state-transition transaction against the published frame.
2. Enrich pointer-up click count from the prepared click target without another hit query.
3. Verify and commit the prepared surface input transition, then execute handlers/default actions.
4. Apply dispatch effects and remaining surface input state changes.
5. Build compact scalar execution facts and move the routed bubble path into the manager envelope.
6. Update active hover, press and capture state from the execution facts.
7. Update cross-surface capture ownership from the same capture scalar.
8. Evaluate the capture policy once.
9. Only when selected, clone/materialize route trace, popup IDs, source notes and route steps into
   the public diagnostics DTO.
10. Return the public result. The diagnostic snapshot is observational and cannot affect steps
    1-7.

If a full snapshot is required after the active-pointer transfer, diagnostic materialization may
borrow the route before the transfer or clone exactly once into the snapshot. It must not reconstruct
routes by walking the tree or hit grid after dispatch.

## Correctness invariants

- `Disabled` and `Full` produce identical reply, component events, binding reports, focus,
  capture, pressed state, hover transitions, dirty nodes and damage.
- Captured diagnostics preserve the exact physical `UiVirtualPointerPosition` supplied by the
  original query, including control-anchored popup affine projection cases.
- A capture released by pointer-up/cancel remains available to the execution envelope long enough
  to update cross-surface ownership; diagnostic capture timing cannot erase it.
- Popup stack order and IDs in a full trace represent the same published surface frame used for
  hit testing. No event-time arranged/render traversal is allowed.
- Sampling and error selection are deterministic and side-effect free.
- A primary pointer release performs one hit query. The prepared click target, dispatched route and
  double-click candidate all come from that query and one frame generation.
- The normal pointer-move path has zero route diagnostic vector allocations, zero diagnostic
  popup string clones and zero diagnostic note string allocations.

## Counters

The migration is not accepted without source-bound counters for:

- input events by kind and capture policy;
- trace selected/skipped count and selection reason;
- trace route node copies, route-step rows and popup string bytes;
- execution hover path moves and unexpected execution-path clones;
- hit queries and any event-time arranged/render scans;
- prepared-route commits, generation retries and duplicate-hit violations;
- pointer dispatch CPU duration and allocation count/bytes;
- active-pointer table updates, capture transitions and hover enter/leave counts;
- RSS samples during sustained pointer movement.

Counters must be bounded integer/scalar writes. They must not allocate labels or format strings on
the input hot path.

## Test-first implementation plan

1. Add a lower-layer route-transaction test proving primary release performs one hit query, click
   count is visible to handlers, and a stale generation produces one typed retry without mutation.
2. Add a lower-layer equivalence test that dispatches the same pointer sequence under `Disabled`
   and `Full` and compares all semantic state/results except diagnostic payloads.
3. Add an allocation/source contract proving 10,000 normal pointer moves select zero traces and
   create zero route vectors, route steps, popup ID copies and note strings.
4. Add a manager test proving the hover table receives the routed path when the public diagnostic
   trace is absent. Add the corresponding capture and cross-surface ownership tests.
5. Add a projected-popup test with a non-1:1 affine transform proving frame hit and instance hit
   agree, the old placeholder rejects, and captured diagnostics retain the original physical
   virtual pointer.
6. Add `ErrorsOnly` and deterministic `Sampled` tests, including a blocked/rejected dispatch.
7. Introduce the internal execution envelope and convert all production reads of diagnostic route
   state. Keep full capture enabled in existing route-contract fixtures until they explicitly opt in.
8. Gate route trace, route-step, popup ID and informational-note materialization behind the policy.
9. Add the real Editor product-path pointer stress and compare `Disabled` with `Full` using managed
   validation and a source-bound executable.

## Stress and acceptance

Use a 10,000-node retained tree with route depths of 4, 16, 64 and 256; include no popup, one popup
and a four-popup stack. Run at least 10,000 pointer moves after warm-up, plus down/up/cancel,
capture transfer and popup-affine cases. Record median and p95 event latency, CPU time, allocation
count/bytes and RSS for `Disabled`, `ErrorsOnly`, `Sampled` and `Full`.

Acceptance requires:

- `Disabled` normal moves report zero diagnostic route-node copies, route-step rows, popup string
  bytes and note-string allocations;
- `Disabled` p95 grows with route execution depth only, not popup diagnostic string volume;
- semantic state and event results match `Full` for every test sequence;
- full captured traces match the existing route contracts exactly, including virtual pointer and
  popup order;
- sustained pointer movement does not produce monotonic RSS growth after warm-up;
- no event-time scan of arranged nodes or render commands appears in source guards or counters;
- the real Editor button/resize interaction capture demonstrates the improvement with the same
  source hash, workload and managed validation receipt.

The product result remains unmeasured until a source-bound `zircon_editor.exe` can be built through
the official managed validator. Static source evidence is sufficient to prioritize this migration,
not to claim that mouse interaction is fixed.
