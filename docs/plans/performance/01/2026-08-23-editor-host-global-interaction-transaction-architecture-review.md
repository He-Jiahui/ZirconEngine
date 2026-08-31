---
title: Editor host global interaction transaction performance review
date: 2026-08-23
module: zircon_editor retained-host host_contract globals
priority: MVP-P0 editor input publication, generation coherence and redraw suppression
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate invalidation root and widget proxy
---

# Goal

Publish one coherent interaction transaction per native fact or retained-state writeback. Stable
facts must not clone the complete interaction DTO, advance several generations, copy a wide host
presentation before discovering no change, or request redraw for a duplicate viewport resource.

## Reviewed source

- pre-M0 owner Rust files: 16/16
- pre-M0 physical lines: 1,318
- pre-M0 bytes: 60,518
- pre-M0 path-and-file-SHA manifest SHA256:
  `57f44389363d0a1905c380a7a0ada2c6f8dc3b6525f33dbd42dc555b00e3ad6a`
- post-M0 owner Rust files: 16/16
- post-M0 physical lines: 1,551
- post-M0 bytes: 70,766
- post-M0 path-and-file-SHA manifest SHA256:
  `d8ecbfb7ca0708ab8ab6b24e75cb4623093a6433563a564186d5288392815aa6`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Owner scope is `host_contract/globals.rs + globals/**`. All files were read in full. Direct callers
in asset/hierarchy pointer writeback, template hover, drag/resize, viewport image polling, viewport
toolbar chrome projection, presentation generation capture and callback dispatch were traced.

The current working tree already contains narrow drag/resize accessors and an interaction-generation
accessor in `state.rs`/`ui_context.rs`. They are preserved and treated as current-source foundations;
this review does not overwrite or attribute those edits. The 2026-07-17 report is stale: this owner
grew from 600 to 1,318 lines and now owns split Arc generations, hit indexes and patch APIs.

## Correct foundations to retain

1. Structure, interaction, viewport, hit-test and diagnostics have separate generation counters and
   retained Arc payloads. Event callers can capture one `HostPresentationGeneration` cheaply.
2. Workbench hit-index rebinding is coupled to structure patches instead of being rediscovered by
   every event.
3. Callback invocation clones only the `Rc<dyn Fn>` and releases the `RefCell` borrow before calling,
   preserving reentrancy and preventing callbacks from running under the host-state borrow.
4. Stable menu, overflow, text-focus, viewport and diagnostics replacement checks identity/value
   before advancing their generation.
5. Drag/resize now have narrow hot-path accessors, avoiding several whole-DTO clones during move.
6. The state is UI-thread-owned `Rc<RefCell<_>>`; adding worker threads around it would violate the
   ownership model and would not solve the current transaction amplification.

## Structural findings

### P0: one asset writeback becomes eight interaction transactions

`apply_asset_pointer_state_to_ui` invokes eight field setters for either activity or browser state.
Each setter enters `update_pane_interaction`, clones all 26 fields including four owned Strings,
compares the complete DTO, publishes an Arc and advances the interaction generation if that one
field changed. One pointer move/scroll/click can therefore perform eight full clones/comparisons and
publish an observable sequence of partial states. Hierarchy writeback repeats the same pattern twice.

M0 adds event-level asset and hierarchy setters. They compare all addressed fields before mutation,
then publish at most one interaction generation. Existing narrow setters remain for genuinely
independent operations and tests.

### P0: no-op interaction setters still clone the complete DTO

`HostContractState::update_pane_interaction` clones before the equality check. Stable writebacks avoid
generation advancement but still pay the clone/compare cost. M0's transaction setters add a borrowed
precheck, making stable asset/hierarchy writeback clone-free. M1 replaces ad hoc setters with a typed
`HostInteractionPatch` that carries changed domains and applies copy-on-write only after a no-op test.

### P0: viewport chrome performs wide copy-on-write before detecting no change

`patch_scene_viewport_chrome` calls `Arc::make_mut(&mut host_presentation)` before it compares scene
pane and status fields. While a paint/input generation retains the previous Arc, a stable toolbar
projection can clone the complete host presentation and its nested DTO graph, then return `false`.

M0 adds a borrowed preflight across docked/floating Scene panes and the two status nodes. The wide
copy-on-write happens only when at least one projected field differs. M1 moves viewport chrome to a
dedicated retained subgeneration so a real toolbar change does not clone unrelated shell data.

### P0: duplicate captured viewport resources report a false update

`set_viewport_product` returns `replace_viewport_image`'s boolean, but `set_viewport_capture` ignores
it and always returns `true` after conversion. A repeated viewport/generation resource key therefore
requests paint-only invalidation and a viewport redraw despite no state change. M0 returns the actual
replacement result for both ingestion paths.

### P1: generic presentation mutation cannot prove no-op or dirty domains

`update_host_presentation` gives a mutable reference to the whole presentation, then always checks/
possibly rebuilds the hit index and advances structure generation. Callers cannot report which
domain changed. M2 hard-cuts generic mutation in product paths to typed structure/geometry/resource
patches with expected generation and exact dirty-domain receipts.

### P1: callback contracts own Strings and erase mutation effects

Dozens of callbacks accept owned `SharedString` (`type SharedString = String`) and return `()`. Route
ids and labels are copied across a generic callback boundary, while dispatch must infer redraw/frame
effects afterward. M2 introduces typed stable ids and replies carrying handled state, interaction/
structure effects and exact damage owners. The callback `Rc` clone remains because it is required for
safe reentrant invocation.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
  - `InvalidateWidget` ORs reason bits and uses `HeapPushUnique`, `PushBackUnique` and
    `PushBackOrHeapUnique`; repeated invalidations accumulate into one owner entry.
  - `ProcessInvalidation` processes the retained pre/attribute/prepass/post queues as one update pass
    and resets the queues afterward; the slow path is explicit.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
  - `FWidgetProxy::Update` repaints only widgets carrying repaint flags; layout propagation occurs
    only after desired-size/visibility evidence requires it.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
  - `SWidget::Invalidate` normalizes reason dependencies, then hands the retained proxy to the
    invalidation owner instead of immediately rebuilding the complete window state.

The transferable rule is one owner-level invalidation transaction with accumulated reasons and one
retained processing pass. Zircon should not copy Slate's widget objects, but it should not expose
eight partially published host generations for one logical pointer-state writeback.

## Target architecture

1. Build one typed `HostInteractionPatch` per native fact/writeback with menu, overflow, pane,
   focus, drag, resize and capture domains plus exact changed fields/owners.
2. Preflight the patch against the current immutable generation; stable patches allocate nothing and
   publish nothing.
3. Apply copy-on-write once, advance one interaction transaction id and publish all changed domain
   Arcs/hit/damage metadata atomically.
4. Give structure/viewport/diagnostics independent typed patch APIs and remove generic product-path
   mutation.
5. Return a typed callback reply containing handled state, changed generations, dirty owners/regions
   and frame-update need.
6. Instrument transaction fields, clones, copied String bytes, Arc strong counts, generations and
   redraw suppression reasons.

## Instrumentation and acceptance

Matrix: event `move/scroll/press/release`; owner `asset/hierarchy/template/menu/drag/resize/viewport`;
state `stable/one-field/eight-field`; retained readers `0/1/8`; rate `1/125/500/1,000 Hz`; rows
`1/100/10K`; backend `GPU/softbuffer`; scale `1x/1.5x/2x/4K`.

Acceptance requires:

- asset and hierarchy writeback transactions per logical call <= 1;
- stable transaction DTO clones, String copied bytes and generation advances = 0;
- one changed asset snapshot advances the interaction generation exactly once;
- duplicate captured viewport resource causes no generation advance and no redraw;
- stable viewport chrome performs no `HostWindowPresentationData` copy-on-write;
- callback invocation remains reentrant and never holds the state borrow across foreign work;
- final p95 interaction publication below 0.05 ms at 1,000 Hz on the recorded host;
- input behavior, hover/scroll, hit/paint generation coherence and pixels remain equivalent.

WPR owns UI-thread CPU, allocations, wakeups and package-energy evidence. RenderDoc is used only after
a current-source GPU executable exists, and only for draw/scissor/resource/pixel parity. Artifacts
and targets remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Batch asset/hierarchy writes; preflight stable viewport chrome; return exact capture update. | applied; focused RED 0/5 to GREEN 5/5 |
| M1 | Add typed event-level interaction patch and one atomic publication transaction. | zero stable clone; one generation/event |
| M2 | Hard-cut generic product mutation and void/String callback contracts. | typed dirty domains, stable ids and replies |
| M3 | Run storm/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 16/16 current Rust files.
- Direct writeback, input, generation, viewport and callback callers: read and mapped.
- Unreal invalidation-root/widget-proxy/widget implementation: read and mapped.
- M0 implementation: applied. One named asset interaction snapshot replaces eight scalar setter
  transactions; hierarchy uses one two-field transaction; both preflight the retained state before
  the one full DTO update. Stable viewport chrome returns before `Arc::make_mut`, and capture returns
  the exact resource replacement result.
- Worst-case logical writeback publication count is statically reduced from 8 to 1 for asset state
  and from 2 to 1 for hierarchy state. Stable asset/hierarchy writes now perform zero complete DTO
  clones and zero generation advances. These are code-path counts, not elapsed-time claims.
- Rust regression source verifies one interaction-generation advance followed by zero for a stable
  asset writeback, retained presentation Arc identity for stable chrome, and duplicate capture
  returning false. The source is formatted but not claimed passing until managed Cargo runs.
- Focused static contract:
  `tools/tests/test_editor_host_global_interaction_transaction_performance_contract.py`, 79 lines,
  2,934 bytes, SHA256
  `53dd97b15b500ec112a031e3b0509f9a5d08ccbbca2df9adf3a2c33d3ce438b3`; RED 0/5 to GREEN 5/5.
- Adjacent asset-pointer/native move/scroll/routing/drag/window-resize contracts: GREEN 26/26.
  Rustfmt and scoped `git diff --check` passed; no `too_many_arguments` exception remains.
- Broad performance static discovery: 180/186 passed. The six failures are the existing external
  drift: two removed test-fixture paths, missing `available_slots`, two UI-surface root clones and
  Runtime 07 source/telemetry/owner-gate documentation. No globals contract regressed.
- Owner size increased by 233 physical lines and 10,248 bytes because the named transaction,
  read-only chrome preflight and three Rust regression sources are explicit. The direct asset caller
  removed the duplicated activity/browser eight-setter branches; M1 must not grow another parallel
  patch representation.
- Managed Rust tests, M1-M3, current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M3 pass on one source/executable fingerprint.
