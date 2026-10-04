---
title: Editor native pointer move transition generation performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer move_dispatch
priority: MVP-P0 editor idle-hover, drag and resize input latency
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate retained previous/current pointer path routing
---

# Goal

Route each native pointer sample through one committed generation, materialize only the winning typed
hit, and publish one compact transition receipt. Stable hover must not clone strings or wide state,
drag/resize capture must not pay idle-hover instrumentation, and target callbacks must not publish
multiple interaction generations for one input fact.

## Reviewed source

- owner Rust files: 17/17
- lines: 537
- bytes: 19,374
- source-only SHA256 over lexicographically sorted owner files:
  `1fa72c467b75fb6d3c7b12ac61d584a18ec60031d8df2b4103e526ca2cc764e6`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

Owner scope is `native_pointer/move_dispatch.rs + move_dispatch/**`. All files were read in full.
Direct boundaries read include native drag/resize capture, menu/page-overflow geometry, pane/workbench
routing, `TemplateNodePointerHit`, workbench hit index, pane interaction state/generation replacement,
template-hover setters, redraw results and callback consumers. The current owner had no local diff at
review start.

## Correct foundations to retain

1. Resize and tab-drag capture run before ordinary menu/workbench/pane routing and can consume the
   sample. Popup workbench hits retain priority over underlying panes.
2. The body reads one committed presentation generation and passes its structure/menu/overflow/hit
   artifacts through the route chain. Page-overflow unchanged rows and menu unchanged interaction
   generation return idle.
3. Workbench/pane hit testing uses retained indexes and arranged artifacts rather than scanning all
   nodes on every move.
4. Pointer target dispatch is typed at the top-level enum, and viewport movement can avoid native
   repaint when no overlay hover state changes.

## Structural findings

### P0: stable hover allocates before discovering equality

`TemplateNodePointerHit` is an owned record with eleven Strings. Workbench hover clones control,
dispatch, action and value strings on every sample before calling the setter. The setter then clones
the complete 26-field pane interaction state, including four Strings, mutates the copy, and only then
compares it to the current state. A stable popup-row hover can therefore allocate/copy up to eight
Strings in this path without publishing a new generation.

M0 changes template-hover setters to borrowed inputs with a pre-mutation equality gate, so strings
and the immutable interaction snapshot are cloned only on a real transition. M1 replaces the wide
owned hit with a compact indexed `PointerTransitionReceipt` and resolves extended action/edit/table
payload only for the consumer that needs it.

### P0: the committed before generation is discarded and reacquired

The body already owns the generation used for routing. Pane and workbench dispatch nevertheless call
`get_host_presentation_generation()` again for before and once more for after. Each read clones all
generation Arcs and increments read telemetry; `clear_hovered_template_move` instead deep-clones the
26-field pane state twice. Stable input pays snapshot traffic unrelated to the changed interaction.

M0 passes the body generation into pane/workbench dispatch and reads only one `Arc` interaction
snapshot after the callback. Clear uses two narrow Arcs. M1 makes the callback return its transition
receipt directly, eliminating after-state rediscovery.

### P0: workbench hit materializes before underlying pane priority is resolved

Outside menus, the body creates an owned workbench hit before asking the pane route because popup
precedence must be known. If the hit is non-popup and a pane wins, the wide hit and its strings were
materialized but unused. Current indexed candidate selection is the right foundation, but the result
needs a cheap typed header/row id first and lazy extended payload projection second.

M1 makes the hit index return a borrowed/indexed candidate with family/role/dispatch flags. Popup
priority and pane competition use that header; only the winning route materializes action payload.
This is shared with the surface-hit generation M1 and must not create a second index.

### P0: asset/reference routes and callbacks own strings per sample

Asset tree/content movement clones one mode String; reference movement clones mode and list kind.
Reference movement can first publish the hover frame and then publish the callback-updated row index,
creating two pane-interaction generations for one sample. String modes also encode a closed
activity/browser category.

M1 uses typed asset surface/list identities and one batched interaction patch containing frame,
old/new row and scroll facts. Stable target/row emits no publication.

### P1: capture is measured as idle hover

`UiPerfScenario::IdleHover` guards are created before capture routing, while drag/resize handlers open
their own scenarios. Captured samples are therefore nested and attributed to idle hover as well as
drag/resize, distorting the baseline and adding avoidable timer work. M0 moves idle-hover guards after
the capture early return.

### P1: result damage still promotes to one pane/bounding frame

Ordinary non-template pane changes can fall back to the complete pane frame, while combined hover
changes union disjoint frames. The separate redraw-result region-promotion record owns typed effects
and shared multi-region transport; this move owner supplies transition receipts and does not invent a
damage container.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/DragAndDrop.h`

Slate locates one retained widget path per pointer move, compares it with the user's previous weak
path, emits enter/leave only on path transitions and routes move/drag-over along the winning path. A
stateful drag operation is retained across samples. The transferable invariant is one path snapshot,
one transition and lazy handler-specific work. Zircon should retain its hit grid and typed Rust
routes, not copy Slate widget objects or infer Unreal timing values from source.

## Target architecture

1. One committed arranged/presentation generation supplies popup, workbench and pane candidate ids.
2. Hit queries return compact borrowed/indexed headers; extended strings/action/table payloads are
   materialized only after the winning route is known.
3. Pointer handlers produce one typed transition receipt and one batched interaction patch.
4. Stable point/target/row performs zero String allocation, zero state clone/publication and no redraw.
5. Capture and idle-hover scenarios are exclusive and expose route/hit/callback/state/damage stages.

## Instrumentation and acceptance

Matrix: pointer `125/500/1000 Hz`; samples `1/1K/1M`; nodes/panes/windows `1/100/10K`; route
`capture/menu/overflow/popup/pane/workbench/none`; target `stable/adjacent/cross-pane`; asset mode/list;
backend `GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

| Evidence | Acceptance |
| --- | --- |
| generation/Arc reads and state clones | one before generation; no stable after/state clone |
| hit candidates/materialized fields/String allocations | losing/stable route: zero extended payload allocation |
| callbacks/interaction generations | at most one publication per transition |
| capture versus idle scenario samples | mutually exclusive, attributable counts |
| route/hit/callback/damage CPU p50/p95/p99 | slope bounded by candidate/path depth, not total nodes |
| redraw useful/union/submitted area, WPR scheduling/power | exact source/workload fingerprint |

RenderDoc is used only for eventual GPU scissor/draw/pixel parity. WPR owns CPU, allocation,
context-switch and power evidence. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Move capture before idle timing; borrow stable hover inputs; reuse before generation and narrow after state. | applied; static contract GREEN, managed Rust/dynamic pending |
| M1 | Add compact indexed hit headers, typed asset identities and one transition/batched patch. | stable/losing move zero allocation/publication |
| M2 | Integrate typed receipts with exact multi-region redraw. | no full-pane/bounding fallback without reason |
| M3 | Hard-cut wide owned hot-path hit/string mode and after-state rediscovery APIs. | one generation/transition authority |
| M4 | Run input-scale WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## M0 implementation result

Capture routing now returns before `IdleHover` scenario/timer creation, so drag/resize samples pay only
their owning instrumentation. Pane and workbench routes reuse the body generation as the before
snapshot and fetch one narrow interaction `Arc` after dispatch. Their previous two additional full
presentation-generation reads are removed. Menu compares a scalar interaction generation after its
callback; clear retains two interaction Arcs instead of deep-cloning the 26-field state twice.

Template hover setters now accept borrowed strings/frames and compare the current immutable state
before calling `update_pane_interaction`. Stable popup-row hover no longer performs the four caller
String clones plus four String copies from cloning the current interaction state in this setter path.
Actual transitions retain immutable before/after semantics and clone only the new owned values. The
wide hit itself and asset callback String identities remain for M1.

Post-M0 owner scope:

- Rust files: 17/17
- lines: 529
- bytes: 19,234
- source-only SHA256: `dfa73917a73271d9497e91e96289395fc256080cc38cc3a52441faa474006a99`

Current after routing M0 interaction integration: 17 files, 534 lines, 19,310 bytes, source-only
SHA256 `bfe91d6404d7eac7bd280f567482141c442de0e7707ad21d83b7d119e666888f`.

Supporting changes are limited to narrow interaction accessors in `globals/state.rs` and
`window/presentation.rs`, borrowed/equality-gated hover setters, and their native-keyboard/window-test
call sites. The focused static contract is
`tools/tests/test_editor_native_pointer_move_generation_performance_contract.py`, 60 lines, 2,967
bytes, SHA256 `7081879dc84844e7215e44de04e8cb8354db85156599edafd7b44066661400ef`.

## Validation state

- Owner source review: passed, 17/17 current Rust files.
- Direct capture, menu/overflow, pane/workbench hit, state/generation, setter and redraw boundaries:
  read and mapped.
- Unreal pointer-path and drag-operation sources: read and mapped.
- M0 focused static contract moved RED 0/4 to GREEN 4/4. Native keyboard, popup, template hover,
  page-overflow, hit-route, menu, context-menu and drag/resize adjacent contracts are GREEN 26/26.
- Routing M0 now supplies the captured generation's split interaction state to move pane routing;
  the current owner fingerprint is recorded above and combined adjacent contracts remain GREEN.
- Broad `test_*performance*.py` discovery is 156/162. The six failures remain external drift: two
  missing editor test files, one missing `available_slots` source anchor, two UI asset
  `.roots.clone()` findings and Runtime 07 documentation/source telemetry drift.
- Changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with only
  repository line-ending warnings.
- Existing window generation tests were updated for borrowed setter signatures but are not claimed
  passing until managed Cargo is executable. M1-M4, current-source launch, WPR and RenderDoc remain
  pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
