---
title: Editor host window event-loop and present protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-host-window-event-loop-present-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/window.rs + window/**`
- 43/43 Rust files source-reviewed. Event-driven idle, edge-coalesced wakes, one native redraw per
  pending transition, latest-size resize and bounded surface retry are retained. Pending M0-M5: remove
  indefinite 50ms presenter-readiness polling, give GPU/softbuffer one shared resize transaction,
  remove capture materialization/raster/encode/fsync from measured present, replace long-text full-value
  copying with persistent changed-range edit state, propagate multi-region damage, and run current-source
  WPR/power/RenderDoc acceptance.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP editor startup/idle/input/resize. Record readiness polls/wakes/mutex acquisitions,
resize scene builds/reflows/pixels/bytes, capture self-overhead, text copied bytes/rebuilds, CPU/RSS/p95
latency/context switches/package energy and same-build RenderDoc parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of the 50ms readiness-poll path, the trait default that converts native resize into an
ordinary full present, synchronous measured-present capture compatibility and single-rectangle damage
APIs after their typed replacements are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the window-level resize transaction, generation/damage receipts, stable presentation artifact and
main/native-window scheduling. Retain O(1) native redraw scheduling and final exact layout parity.

## `docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md`

Own interactive resize semantics: snapshot age, at-most-one budgeted reflow per display frame when
needed, one final exact reflow and no backend-specific full scene rebuild storm.

## `docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`

Own typed Job terminal completion delivery and the edge-coalesced event-loop wake used by viewport
render-framework readiness. Polling must not be the normal completion protocol.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own persistent text edit state, caret/selection/IME revisions and changed-range callbacks. Immutable
full values remain binding/commit artifacts rather than the per-character editing authority.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared invalidation generations, bounded damage-region sets and prepared UI render artifacts that
survive window scheduling, retry and presenter consumption.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own GPU/softbuffer resize-transaction counters, presenter readback/capture boundaries and RenderDoc
draw/upload/scissor/GPU/pixel parity. WPR CPU and package power remain owned by Performance01.

## Acceptance handoff

The handoff requires 43/43 post-change fingerprints, focused and managed Rust tests, readiness/resize/
backend/capture/text/damage matrices, same-executable WPR and power artifacts on D/E/F, current-source
RenderDoc GPU parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
