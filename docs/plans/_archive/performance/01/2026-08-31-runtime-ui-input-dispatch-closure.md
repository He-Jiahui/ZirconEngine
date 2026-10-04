---
related_code:
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/surface/mutation_snapshot.rs
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-31-runtime-text-document-layout-closure.md
  - docs/plans/performance/01/2026-08-31-runtime-ui-layout-closure.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
write_scope: []
status: pending
---

# Runtime UI input and dispatch closure

This is a current-source static revalidation of the retained UI input manager,
pointer/navigation dispatch, effect application, text/IME/clipboard paths,
transient timers and input diagnostics. It remains pending because current
Cargo fails in broad foreign integration work and no F4 product profile is
available. No Rust source was changed.

## Scope and source state

- `zircon_runtime/src/ui/surface/input/**`: 84 Rust files, 14,698 physical
  lines, 13,664 nonempty lines, 506,032 bytes, 86 tests, 15 ignored tests and
  15 include sites. Raw-content SHA256:
  `066023ac52a551da103a52f3db69a32d7c41db69f8281713f2112b47f568c7bb`.
- `zircon_runtime/src/ui/dispatch/**`: 34 Rust files, 7,113 physical lines,
  6,543 nonempty lines, 244,048 bytes, 66 tests, five ignored tests and six
  include sites. Raw-content SHA256:
  `58da6fe0b41631560fb6f2c3840ff78bea4ab3191746eb68705647b7d6bd902d`.
- Combined scope: 118 files, 21,811 physical lines, 20,207 nonempty lines,
  750,080 bytes, 152 tests, 20 ignored tests and 21 include sites. Sorted-file
  raw-content SHA256:
  `1caa4dfdf93c01d7664d0d9cf232fe574a74a1ff66fcdaa29cc002ebb1b75810`.
- The focused scope is broad foreign modified/added work. Tracked focused diff
  alone reports 3,905 insertions and 1,309 deletions across 64 files, while
  staged/untracked split files add more foreign ownership. Isolated rustfmt
  passes 105 of 118 files; the other 13 fail current import/assertion
  formatting. Scoped diff-check reports line-ending warnings only. Existing
  work was preserved.
- The current E-drive `zircon_runtime` check fails with 214 broad foreign
  integration errors and 360 warnings. The earlier text-document vector,
  overflow-outcome and constraint-iterator blockers are closed, but graphics,
  runtime-contract, scene, input and feature reexport integration remains red.
  No focused Rust test executed and no product binary, F4, WPR or pixel evidence
  is claimed. The ignored tests are synthetic/source-contract evidence, not
  accepted latency results.

## Product path and positive work

- `UiInputManager` is the common product owner for individual events, batches,
  window pumps and timer ticks. It synchronizes text-document ownership,
  dispatches through `UiSurface`, updates retained pointer/timer state and then
  publishes one result. No second input framework or background dispatcher was
  found in this scope.
- Pointer routing reuses a cached physical route when target identity matches.
  Hovered-path storage preserves capacity, compares before replacing, and the
  dispatch visited set stays inline for the common depth of 16 before promoting
  to a hash set.
- Text-model update queues have explicit 256-row, 16 MiB aggregate and 4 MiB
  value limits with terminal receipts. Unchanged document topology avoids a
  full owner-binding scan, and text constraints sanitize in one pass with an
  exact receipt.
- Number-field parsing is capped at 128 bytes, validates finite policy/value
  state and uses checked edit/value revisions. Common one/two keyboard edit
  actions use inline storage instead of a heap vector.
- Drag payloads are Arc-backed and drag sessions carry stale-generation checks.
  Text document/property preparation is moving toward an explicit candidate
  transaction. These controls should be retained while ownership converges.

## Retained findings

1. **Default diagnostics owns route projections on ordinary input (P0).**
   `UiInputManager::default` selects `UiInputDiagnosticsMode::Full`. Pointer,
   navigation and analog paths can gate some detail, but keyboard, clipboard,
   editable text, accessibility, popup and timer dispatchers call route-policy
   and route-step annotation unconditionally. They clone the input event,
   construct owned phase/authority strings, materialize bubble/focus/preview
   paths and append notes. Summary mode applies a final budget but does not
   structurally prevent all producer work. Full mode is therefore the product
   default, and even Summary remains incomplete as a no-work contract. The
   128-path, 256-step, 32-note, 16-popup and 8 KiB string limits are positive
   output bounds, but several routes are built before truncation and no one
   aggregate proposal admits the event.
2. **Atomic effect rollback clones broad retained surface state (P0).** Any
   reply with more than one effect, plus drag/drop, popup and dismissal
   families, prepares an atomic transaction. Its snapshot can clone the whole
   `UiTree`, runtime-style index, invalidation state, dirty-node set, focus,
   complete `UiSurfaceInputState`, component-state store, navigation state and
   clipboard-transfer snapshot before applying the effects. Effects are also
   cloned from the reply into application and retained again in the result.
   This gives common focus/capture/dirty combinations surface-sized copy work
   even on success. Rollback correctness is valuable, but it must come from
   pure prevalidation and a narrow mutation journal/candidate generation rather
   than a full accepted-state copy.
3. **Focused text and IME updates can rebuild the whole surface (P0).** An
   accepted keyboard/text/IME edit, and selection changes while an IME owner is
   active, call `refresh_render_extract_for_current_tree`. That path rebuilds
   arranged tree, node/slot/visibility indexes, hit test, render extract, popup
   projection and navigation, marks the frame dirty and publishes a surface
   frame. IME context construction then scans the render-command list to find
   the focused text layout. A selection drag can repeat this per pointer move.
   The property transaction also constructs ten owned property names and
   clones full text values for parallel tree/style/component/binding
   projections; each compiled component action can clone the text payload
   again. The existing document transaction is directionally positive, but
   caret/composition geometry needs a retained per-node text-layout generation,
   not input-triggered whole-surface publication.
4. **Timer and owner reconciliation work is event/burst amplified (P0/P1).**
   Every input, window event and timer tick synchronizes document owners. The
   topology-stable path avoids one broad binding scan, but still scans pending
   text-model updates and allocates a rejection vector. Each tick separately
   retains/scans typeahead, submenu, tooltip and toast maps, allocates four due
   vectors and synchronously dispatches every expired row. Timer maps, popup
   stack and tick work have no aggregate count/label-byte/deadline proposal, so
   one expiry burst can monopolize the input owner despite bounded individual
   text updates.
5. **Clipboard ownership duplicates selected payloads and splits terminal
   state (P1).** A copy/cut extracts a new selected-text String. Beginning the
   request clones `UiClipboardRequest` into the dispatch effect/host request
   while the pending transfer retains its own owner/property/revision state.
   Manager translation scans host requests and clones surrounding IME text.
   The clipboard queue is capped at 256 rows but performs an owner-wide linear
   retain for each insertion and has no aggregate pending-byte/deadline lease.
   Count bounds and secure-text policy are positive; payload, callback and host
   completion should belong to one terminal request generation.
6. **High-frequency pointer and analog identities retain avoidable dynamic
   work (P1).** Pointer Up/Cancel clones the complete pointer-capture map to
   temporarily bypass one captor. Pointer-table lookup is linear and active
   pointer/capture count is not admitted. Analog navigation formats an owned
   `user:control:kind` String for each sample and uses it as a BTreeMap key;
   reset builds two more keys. Normal device counts are small, so no measured
   hotspot is claimed, but the public scale path should use checked pointer
   capacity and dense typed control identity.
7. **Input state publication has branch-local escape paths (P1).** Text pointer
   capture can mutate before the property transaction succeeds, while the
   secondary text context-popup route manually changes popup/input state and
   constructs host effects outside the common transaction. These paths are
   concrete instances of split input/surface publication: a failed edit or
   later frame must not leave capture, popup, component, document and rendered
   generations disagreeing.

## Architecture handoff

1. Compile one immutable `UiInputSurfaceGeneration` from accepted tree/layout,
   route, focus, component, text-layout and platform-input identities. Stable
   dispatch borrows dense node/route/control IDs and never rebuilds labels or
   route DTOs.
2. Run one checked `UiInputEventProposal` before callbacks or mutation. It
   admits route/handler/effect/component/host/timer rows, payload and diagnostic
   bytes, owner generation and deadline, producing
   `Ready/Deferred/Backpressured/Invalid/Fault`.
3. Replace whole-surface rollback snapshots with pure effect validation and a
   narrow mutation journal/candidate. Handler, document, property, focus,
   capture, popup and clipboard changes publish atomically with the accepted
   input/surface generation; failure restores no state because none was
   published.
4. Publish one per-node `EditableTextLayoutGeneration` containing document
   revision, line/caret/selection/composition geometry and platform IME
   context. Input edits mark exact layout work; the ordinary accepted layout
   generation updates cached geometry and emits one platform notification.
5. Replace four retain-scanned timer maps with one bounded scheduler generation
   keyed by deadline and owner. Tick admits a count/time/payload batch and
   defers excess work explicitly. Popup/tooltip/toast/typeahead identity and
   cancellation terminalize exactly once.
6. Unify clipboard/IME/component host work under terminal request leases with
   exact owner/frame/payload identity. Reuse one payload slab and reserve queue,
   callback and result capacity atomically.
7. Enforce diagnostics `Disabled/Counters/Sampled/Full` at every producer.
   Disabled emits no event clones, route/step vectors, labels or notes;
   Counters uses dense fixed IDs; Full borrows compiled names and respects the
   event proposal before allocation.

## Evidence and acceptance gates

Unreal Slate routes pointer input over one accepted widget path in
`SlateApplication.cpp` and updates cached text-input-method geometry during the
editable-text widget tick in `SlateEditableTextLayout.cpp:3616-3618`. Its text
layout uses dirty-state updates and targeted block-position refreshes, while
active timers retain explicit handles and pending-execution state. This
supports retained route/text-layout generations, exact IME notifications and
timer identity. Unreal's shared-pointer widget tree, native IME, active-timer
set and fatal assumptions do not prove Zircon's Rust API, scheduler, budgets or
transaction semantics.

M0 adds RED counters for event/route/effect rows, snapshot clone bytes,
whole-surface refreshes, text/property/component payload copies, timer scans and
expiry bursts, clipboard bytes, pointer-capture copies and diagnostics modes.
M1-M4 establish input/surface/text-layout generations and narrow transactions.
M5-M7 establish bounded timers/sidebands/diagnostics and collect current F4
evidence after source gates recover.

Acceptance covers events 0/1/64/1K/cap+1 and bursts; routes depth
0/1/16/128/cap+1; surface nodes 1/100/10K; pointers 1/2/16/cap+1; effects
0/1/2/16/cap+1 with success/reject/panic; text bytes 0/1/256/1 MiB/cap+1;
IME preedit/commit/delete/selection drag; timers 0/1/64/1K/cap+1 with same and
staggered deadlines; popup nesting; clipboard read/write/failure/owner loss;
and diagnostics Disabled/Counters/Full. Report input-owner p50/p95/p99,
route/node/handler/effect/timer visits, tree/map/Vec/String/Arc clones and bytes,
surface refresh/layout/render publications, queue age/backpressure, callback
time and accepted/rebased/cancelled/fault generations.

Hard gates: current source builds; diagnostics Disabled owns zero route/label/
profile work; a successful common event clones no whole surface; failed effects
publish no partial input/tree/document/component state; text and selection
changes never force a whole-surface render-extract rebuild; one input event has
one exact tree/layout/route generation; timer work is count/time bounded and
cannot monopolize the owner; every host request terminalizes exactly once;
stable identities allocate and clone zero; diagnostics match actual work. No
benchmark artifact or local micro-fix is warranted before these ownership
corrections.
