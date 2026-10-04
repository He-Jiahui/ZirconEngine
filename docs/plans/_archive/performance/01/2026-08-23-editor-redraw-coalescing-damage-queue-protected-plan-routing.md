---
title: Editor redraw coalescing and damage-queue protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-redraw-coalescing-damage-queue-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host redraw.rs + redraw/** + redraw_tests.rs`
- 7/7 Rust files source-reviewed. Native scheduling is correctly bounded to empty-to-pending and surface
  retry uses bounded backoff, but Region is one bounding rectangle through dispatch, external state,
  event-loop pending, retry and presenter; coalescing counters omit spatial amplification, merged scenario
  attribution is lossy and invalid frame-update damage promotes full. M0-M4 region-set/propagation/
  attribution/profile/power acceptance remain pending; no false local wrapper was applied.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP input/event-loop redraw. Record OS requests, input/coalesced batches, region count/
bytes, useful/clipped/presented area, amplification/promotion/retry/source, CPU/RSS/latency/context
switches, WPR power and RenderDoc scissor/pixel parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of single-rectangle redraw/presenter/retry APIs and lossy scenario inference after the
retained damage-set contract is authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own exact damage regions and scene generations across pointer dispatch, external redraw, event loop,
presentation and retry while preserving one native schedule per pending batch.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the shared bounded damage-set, invalidation generation and prepared render-range contract used by
Runtime UI and editor event-loop presentation.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own backend multi-region/scissor counters and RenderDoc draw/GPU parity. CPU/WPR evidence remains owned
by the MVP performance plan.

## Acceptance handoff

The handoff requires 7/7 post-change fingerprints, focused and managed Rust behavior tests, the event/
region/update/retry/backend matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc scissor/GPU/
pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged
until then.
