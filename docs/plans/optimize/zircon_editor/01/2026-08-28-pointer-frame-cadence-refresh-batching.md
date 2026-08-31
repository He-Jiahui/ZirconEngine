# Pointer frame-cadence refresh batching review

Date: 2026-08-28

Status: current-source static candidate; focused source contracts pass;
managed Rust and product timing remain pending.

## Outcome

Native redraw requests are already merged before present, but workbench pointer
feedback performs the expensive retained-surface refresh before reaching that merge.
For each changed pointer event, the callback mutates state, calls
`UiSurface::rebuild_dirty`, rebuilds or patches the host projection, and only then
returns a redraw request to the event loop. Continuous slider/range movement and
rapid cross-target hover can therefore execute multiple surface/projection refreshes
for one displayed frame.

This is an update-scheduler ownership error. It is separate from hit-test speed,
pseudo-style map reconstruction, render caching, and damage submission. Improving
those individual stages does not make repeated undisplayed refreshes necessary.

## Original current-source evidence

- `callback_dispatch/workbench/pointer.rs:29-73` routes the pointer and mutates
  tooltip, virtual-row, hover, press, range, text-input, and focus feedback inside
  the native input callback.
- `pointer_feedback.rs:121-142` calls `refresh_after_state_change` whenever a
  candidate produced pending invalidation.
- `EditorWorkbenchTemplateSurface::refresh_after_state_change` calls
  `surface.rebuild_dirty`, derives a host projection workset, and refreshes the
  projection synchronously.
- `host_contract/window/event_loop/redraw.rs:18-28` receives the completed pointer
  result only after that work and converts it into an interactive frame request.
- `redraw.rs:31-41` merges pending redraw requests. This correctly coalesces native
  present requests, but it is too late to coalesce the preceding UI refresh work.
- `commit_interactive_frame_update` subsequently commits pending host work at the
  redraw boundary, so the repository already has the correct frame-owned place to
  drain a mutation journal.

The cited Editor/Runtime production paths were shared dirty files. The original
review did not edit them.

## Boundary decision

Do not delay the complete input event. These domains remain synchronous:

- hit-test and route selection;
- capture, focus, pressed target, drag ownership, and popup dismissal;
- input sequence/timestamp diagnostics;
- command/action dispatch whose semantic effect is externally observable.

Only coalescible visual/value publication moves to frame cadence. Pointer callbacks
write a keyed latest-value journal such as `(node, property) -> value`, record old
and final damage identities, and request one interactive frame. A slider may
replace the same pending value several times; hover may reduce a series of targets
to the previously published target and the final target. Press/release edges remain
ordered entries rather than latest-value replacements.

At the frame boundary, one transaction:

1. validates the input sequence and surface/style generations;
2. drains ordered edges and latest-value writes;
3. resolves pseudo-style deltas once against the final state;
4. calls `rebuild_dirty` at most once per surface;
5. refreshes and publishes the host projection at most once;
6. submits bounded damage for the semantic state represented by that frame.

If a route-affecting mutation changes geometry or hit eligibility, the transaction
must publish the new frame before a later event is routed against it, or use a typed
barrier that flushes the pending transaction. This is a correctness boundary, not a
license to route future input through stale geometry.

## Unreal reference

`SlateInvalidationRoot.cpp` keeps typed widget invalidation work in retained update
structures. `InvalidateWidget` records work, while `PaintFastPath` consumes the
final update list using cached widget/element state. The relevant lesson is the
separation between event-time state invalidation and frame-time update/paint, not
the exact heap/list implementation.

## Deterministic pressure model

`tools/editor_pointer_frame_cadence_refresh_pressure.py` counts refresh-stage
executions for 1,000 changed events on one continuously changing visual property.
It is not CPU or latency timing. Input routing and semantic state writes remain
1,000 in both designs.

| Changed events per displayed frame | Frames | Current surface + projection refreshes | Target refreshes | Reduction |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 250 | 2,000 | 500 | 4.00x |
| 8 | 125 | 2,000 | 250 | 8.00x |
| 17 | 59 | 2,000 | 118 | 16.95x |

At 17 changed events/frame, 941 surface refreshes and 941 projection refreshes are
avoided. The model deliberately does not weight stages or estimate milliseconds;
actual savings depend on pseudo-style, render extraction, projection, allocator,
and presentation costs measured in the product.

Artifact:
`E:\zircon-profiles\editor-pointer-frame-cadence-refresh-pressure-20260828.json`

## Implementation order

1. Add counters joining input sequence to visual journal writes/replacements,
   surface drains, projection refreshes, frame commits, barriers, and presents.
2. Add a surface-owned pending interaction transaction with ordered semantic edges
   and keyed latest-value visual writes. Bound memory and preserve first-old/final-
   new damage identity.
3. Change pointer feedback to mutate synchronous input authority and append pending
   visual work only. Remove direct `refresh_after_state_change` from pointer moves.
4. Drain once from `commit_interactive_frame_update`, before render submission, and
   publish one generation covering all drained input sequences.
5. Add route-affecting barriers for geometry/hit/input changes. Paint-only hover and
   range value changes must not trigger a barrier.
6. Delete the event-owned refresh fallback after parity and stress tests prove the
   frame transaction. A fallback must remain typed/countable during rollout.

## Acceptance

- For 1,000 changed events delivered faster than the display cadence,
  `rebuild_dirty` and host projection refresh each execute no more than the number
  of displayed frame transactions, except typed route barriers.
- Same-target moves produce zero visual journal writes and zero frame work.
- One-property drag publishes the last value in every displayed frame and the final
  release value; no release edge or action is dropped.
- Cross-target hover publishes old/final target damage without painting all
  intermediate undisplayed targets.
- Capture/focus/press/release routes and physical input sequence diagnostics remain
  identical to the synchronous oracle.
- Geometry/hit-affecting changes cannot route a later event against a mismatched
  frame generation.
- Product profiling records event rate, frame rate, journal depth/replacements,
  barriers, surface/projection refresh counts, input-to-frame p50/p95/p99, CPU, RSS,
  allocator deltas, damage pixels, and present count for hover and range-drag cases.

## Validation status

Static source guards and the deterministic Python model run without Cargo. Managed
Rust and Editor product-path validation remain pending official lane authorization
and current-source closure. No product timing claim is made by this report.

## 2026-08-30 current-source convergence

The active candidate has moved the coalescible refresh to frame ownership:

- pointer feedback stages only mutate surface state and return a candidate flag;
- `refresh_pointer_feedback` checks the pending invalidation count, records
  `ui.workbench.pointer.feedback_deferred_count`, and performs no
  `refresh_after_state_change` call;
- dispatch effects promote pending surface state to `WORKBENCH_PROJECTION` even on
  the host error path; and
- frame recompute calls `refresh_prepared_state_change` before applying the
  workbench projection patch.

The combined pointer batch/deferred focused contracts pass 11/11, and the complete static
performance-contract batch passes 1,630/1,630. The deterministic model remains a work-count
model, not product timing: at 1,000 pointer events it reduces surface/projection refreshes
from 1,000 to 250 at four events/frame (4.00x), and to 59 at 17 events/frame (16.95x,
941 refreshes avoided). The earlier intra-event model separately removes 196,608 dirty
summaries and reduces 131,072 stage-owned refreshes to 65,536 for 65,536 events (2.00x).

The production paths are interleaved with other current-source Editor migrations and are
not owned by this guard-convergence slice. This record therefore claims only static current-
source consistency; managed Rust, current-source Editor CPU/allocator/RSS and input-to-present
P50/P95/P99 remain required before product performance is accepted.
