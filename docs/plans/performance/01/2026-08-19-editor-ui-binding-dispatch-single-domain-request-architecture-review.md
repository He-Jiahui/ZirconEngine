---
related_code:
  - zircon_editor/src/ui/binding_dispatch
  - zircon_editor/src/ui/binding
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/workbench/event
tests:
  - zircon_editor/src/tests/host/binding_dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyHandle.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyHandleImpl.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI binding dispatch single-domain-request architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for Inspector mutation and realtime Viewport input; P1 for the remaining
  Animation, Asset, Draft, Docking, Selection and Welcome conversion paths.
- Accounting: retain `zircon_editor/src/ui/binding_dispatch/**` in `pending.md`. Do not add it to
  `review.md` until the single-domain-request cutover, scale counters, rollback parity and F4 product
  trace pass.
- Code disposition: no Rust source changed. The required correction crosses active Editor/Runtime UI
  leases and must delete duplicated request authorities instead of locally hiding clones.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/binding_dispatch/**` | 35/35 | 1,232 | 0 in-module | 45,551 | `4df9aa1d4115bafdaaf3dd686d84d52b41904b001cf85c76b94b32609b69b91c` |
| external binding-dispatch tests | 8/8 | 1,299 | 26 | 46,629 | `d3b415b666ae04ff9d7747b5facfc06841d396e92cbd9d731eb22dc4977fbcec` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 35 production files
and all 8 external test files were read in full. The current worktree contains foreign changes in
`editor_event_normalization.rs`, root `mod.rs` and six external test files. The production changes are
rustfmt ordering only; the test changes additionally update owned-string calls and Viewport highlight
assertions. This record describes the current worktree and does not absorb those changes.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| `animation` | 3 / 278 | Clones string-heavy binding data into `AnimationHostEvent`, then immediately moves it into the same-shaped final `EditorAnimationEvent`. Track paths are parsed during dispatch rather than compiled at binding-generation ingress. |
| `asset` | 6 / 130 | Clones locators, labels and kind strings into `AssetHostEvent`, then immediately moves them into `EditorAssetEvent`. The intermediate host DTO has no independent authority. |
| `docking` | 4 / 100 | Produces the final `LayoutCommand` directly and is structurally better. Stable IDs and mode values still arrive as parsed binding payload rather than a compiled route descriptor. |
| `draft` | 4 / 72 | Product execution performs `binding -> DraftHostEvent -> EditorDraftEvent -> binding -> DraftHostEvent -> state`, copying subject, field and value on both sides of the event boundary. |
| `inspector` | 6 / 220 | A complete change batch is cloned into `InspectorBindingBatch`, moved into `EditorInspectorEvent`, cloned back into a binding, then cloned again before mutation. Existing checkpoint, selection and transaction rollback behavior is valuable and must move intact to one typed mutation owner. |
| root normalization/error/export | 3 / 298 | Central exhaustive conversion makes the duplicate DTOs look architectural. It should classify once and construct the final domain request, not normalize through another owned host layer. |
| `selection` | 3 / 38 | Produces a compact final host event directly. `apply_selection_binding` has no production caller and keeps an obsolete wrapper alive through tests. |
| `viewport` | 3 / 41 | Serialized binding dispatch clones `ViewportCommand`, maps it to `EditorViewportEvent`, and execution maps it back to `ViewportCommand`. Native retained pointer routing was verified separately to enter `EditorViewportEvent` directly; only its later event-to-command remap remains on that hot path. |
| `welcome` | 3 / 55 | Low-frequency direct host dispatch with no independent MVP bottleneck after the common ownership cleanup. |

## Structural bottlenecks

### P0: normalization duplicates ownership instead of selecting one domain request

`normalize_editor_event_binding()` first recognizes a typed payload, but Animation, Asset, Draft and
Inspector dispatchers allocate another owned DTO before an exhaustive match immediately moves the
same fields into the final `EditorEvent`. Static review found 52 `.clone()` calls and 22
`.to_string()` calls in only 1,232 lines. Those counts are symptoms; the defect is that
`*HostEvent`, `Editor*Event` and binding payloads all model the same request at adjacent stages.

The target is `CompiledBindingInvocation -> DomainRequest` exactly once. External binding decode may
own dynamic data once. In-process typed callbacks should construct or borrow the final request
directly. There must be no generic host DTO whose sole consumer immediately remaps every variant.

### P0: Inspector copies the entire batch three times around one transaction

The current product chain is:

`Inspector binding -> InspectorBindingBatch -> EditorInspectorEvent -> new Inspector binding ->`
`InspectorBindingBatch -> checkpoint/transaction/state`.

Both normalization and execution clone `subject_path` and the entire `changes` vector; the apply
wrapper dispatches the reconstructed binding again. Cost therefore scales with fields and value
bytes before any useful reflection or mutation begins. The eight external Inspector tests correctly
protect harder properties: stale-selection recovery, late unsupported fields, transaction failure,
post-apply selection synchronization failure, active-gizmo refusal, checkpoint restoration and undo.

Do not replace this with a fast but weaker setter. Optimize05 remains the owner of one typed
`PropertyEditRequest`, exact changed property paths, per-target before/after state, one transaction
and immutable edit-session identity. The checkpoint and rollback lifecycle must consume that same
request; execution must never reconstruct a binding to reach it.

### P0: realtime Viewport input still crosses an event-command-event-shaped boundary

The native retained pointer callback already maps structured `UiPointerRoute` directly to
`EditorViewportEvent`; it does not pay native binding formatting or the seven-family decoder. After
that good ingress, `execute_viewport_event()` exhaustively creates a `ViewportCommand` only so state
can match it again. The serialized binding route is worse: `ViewportCommand -> EditorViewportEvent ->
ViewportCommand`.

Optimize49 owns the final correction: realtime input stays in an interaction owner, is coalesced at
frame boundaries with edge/barrier ordering, and does not visit command registry, transaction,
journal or listener registry. One canonical typed input intent should be accepted directly by
Viewport state. Low-frequency grid, snap, projection and overlay commands may remain command intents,
but must not force pointer move through that control-plane enum.

### P1: Draft, Animation and Asset repeat the same architectural mistake

- Draft converts to an event, serializes itself back to binding form and dispatches again before
  changing state. It should become the same typed property-edit authority used by Inspector where
  semantics match, with field/control mapping compiled once.
- Animation duplicates a 97-line `AnimationHostEvent` and a same-shaped final event. Parse graph,
  state, layer and `AnimationTrackPath` identities at authoring or binding-generation ingress, then
  pass one request owner.
- Asset clones owned strings into an intermediate host event that adds no validation or lifecycle.
  Construct the final request and transfer ownership once.

### P1: tests preserve obsolete wrappers but do not measure the algorithm

The 26 tests provide strong semantic coverage, especially for Inspector rollback and Viewport render
state. Several assert intermediate `AnimationHostEvent`/`AssetHostEvent` APIs, while
`apply_selection_binding()` and `apply_viewport_binding()` have no production caller. No test counts
payload clones, copied bytes, conversion passes, reflection parses, event-to-command remaps or
125/500/1,000 Hz behavior. Migrate tests to observable domain receipts and state, then delete the
test-only wrappers rather than preserving them as compatibility facades.

## Reference-engine evidence

- Unreal `UICommandInfo.h` and `UICommandList.h` use a shared command identity mapped to an action;
  invocation does not manufacture a second same-shaped event object before executing the action.
- Unreal `SlateApplication.cpp:5761-6055` routes structured pointer events through the widget path and
  calls move/touch handlers directly. This supports Zircon's existing typed retained pointer ingress
  and rejects routing realtime motion through serialized Editor commands.
- Unreal `PropertyHandle.h:351-416` exposes typed `SetValue` overloads and per-object value operations.
  `PropertyHandleImpl.cpp:542-665` keeps pre-change notification, interactive state, post-change
  notification and transaction end in the property mutation implementation. This supports one typed
  Inspector mutation authority that owns notification/transaction/rollback, rather than a chain of
  binding, host-event and editor-event DTOs.

These sources establish ownership and flow. They do not establish a Zircon timing target by
themselves; same-hardware counters and product traces remain required.

## Required architecture cutover

1. Extend the compiled binding generation from the companion binding review so each invocation
   resolves once to a typed domain request constructor. Local typed callbacks bypass external codecs.
2. Delete `AnimationHostEvent`, `AssetHostEvent`, `DraftHostEvent` and `InspectorBindingBatch` after
   their tests target final domain behavior. Transfer dynamic data once or borrow shared immutable
   payload where lifetime permits.
3. Route Inspector and compatible Draft edits into Optimize05's single `PropertyEditRequest` owner.
   Preserve preflight, selection context, checkpoint, transaction, exact rollback, undo and
   post-apply synchronization in that owner.
4. Split realtime Viewport input from low-frequency Viewport commands. Let Viewport state consume one
   canonical typed input intent directly; remove the `EditorViewportEvent -> ViewportCommand` remap.
   Optimize49 remains the receipt/audit boundary.
5. Compile Animation track/symbol identity and Dock/control IDs per binding generation. External text
   validates once under budget; stable dispatch performs no repeated parse.
6. Remove production-unused `apply_selection_binding()` and `apply_viewport_binding()` once behavior
   tests use the product execution path. Do not leave re-export or shim APIs after hard cutover.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| conversion | every binding family; requests `1/100/10k`; dynamic payload `1 B/1 KiB/1 MiB` | exactly one classification and one final request construction; intermediate host DTO count `=0`; copied bytes equal only the final owned payload |
| Inspector | fields `1/100/10k`; values `1 B/1 KiB/1 MiB`; single/multi-select; success/late failure/post-sync failure | batch deep clones after admission `=0`; reflection parse/resolve once per compiled generation; one transaction; checkpoint, rollback, selection, undo and diagnostics parity |
| Viewport | `125/500/1000 Hz`, 1M move/scroll/edge events, resize/focus/capture barriers | event-command remaps `=0`; command registry/transaction/journal/listener visits for realtime move `=0`; bounded coalescing; all edges and barriers ordered; input-to-damage p50/p95/p99 recorded |
| Animation/Asset/Draft | items `1/100/10k`, strings `1 B/1 KiB/1 MiB`, stable/reload/error | host DTOs `=0`; stable identity parse `=0`; changed request owns payload once; unload/stale/error behavior preserved |
| product | F0 startup and F4 Editor, cold/warm/idle/storm, 31 runs | WPR/ETW CPU, waits, wakeups, allocation bytes, RSS and package power on identical hardware/assets/settings; artifacts remain on D/E/F |

RenderDoc is conditional. It is required if the Viewport cutover changes submitted geometry,
resources, overlays or visible output, with draw/event/resource and pixel parity. WPR/ETW,
allocation counters and domain counters own the CPU conversion proof.

## Static gates executed

- Read 35/35 production files and 8/8 external test files; reproduced 1,232 production lines,
  45,551 bytes, 26 external tests and production fingerprint `4df9aa1d...`.
- Traced normalization into Draft, Inspector and Viewport execution, plus the direct retained pointer
  ingress. Confirmed production-unused Selection/Viewport apply wrappers through caller search.
- Read the cited Unreal command, Slate pointer and PropertyEditor primary sources and the current
  Optimize05/Optimize49 owner reports.
- `rustfmt --edition 2021 --check` passed for all 35 production and 8 external test files. Scoped
  `git diff --check`, 38/38 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed. The production
  fingerprint was recomputed after the documentation write and remains `4df9aa1d...`.
- `python tools/tests/test_editor03_scene_transaction_hardcut_contract.py -v` passed the
  binding-dispatch-relevant Viewport transaction-owner guard and 10 other guards, but finished 11/13.
  Its two failures are stale source-location assumptions: it still expects EditorState construction
  inside `zircon_app/src/entry/entry_runner/editor.rs`, and searches root `transaction.rs` for
  `begin_exclusive_transition` after that method moved to
  `core/editing/engine/transaction/lifecycle.rs`. The protected contract owner must update those
  paths; this review does not weaken the assertions.
- The documentation convention gate reports 0 violations owned by the four binding and
  binding-dispatch records. The unrelated repository baseline remains 692 violations across 242 of
  2,714 documents.
- Dynamic Cargo, clone/allocation counters, rollback scale matrix, F0/F4 launch, WPR/ETW, package
  power and RenderDoc parity remain pending. This is not an accepted milestone, so no commit or WeCom
  notification is due.
