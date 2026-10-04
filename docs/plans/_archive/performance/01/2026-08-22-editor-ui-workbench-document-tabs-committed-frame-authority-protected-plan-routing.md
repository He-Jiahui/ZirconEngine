---
title: Document tabs committed frame authority protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-document-tabs-committed-frame-authority-architecture-review.md
---

# Protected plan updates

This file records requested owner-plan changes without overwriting shared ledgers or numbered plans.

## `docs/plans/performance/pending.md`

Keep one concise folder-level entry:

`zircon_editor/src/ui/workbench/document_tabs` - 2/2 Rust files source-reviewed and static-format
passed; pure metrics are `O(1)`/allocation-free, but committed frame authority, managed tests,
current-source WPR/allocation/power evidence and real-window parity remain pending.

Do not add the folder to `review.md` until the architecture record's M0-M4 gates pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Add or update the MVP editor chrome item to require one generation-keyed document-tab strip layout
shared by projection, pointer and drop. Baseline title-measure/layout-build/node-build counters before
implementation. Steady unchanged frames and drop routing must perform zero title measurements.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Route the hard cutover as a single-authority migration: remove duplicate tab geometry construction
after all production consumers use the committed layout artifact. Do not preserve a compatibility
width estimator in production.

## `docs/plans/zircon_editor/editor_ui/08`

Own the componentized Workbench projection and retained pointer/drop consumer migration. The
`BuiltinWorkbenchWindowLayoutFrames` contract should carry the committed tab layout generation or a
stable handle to the layout artifact, not merely a broad strip rectangle.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Keep `workbench/document_tabs/metrics.rs` as the pure chrome policy owner. Component standardization
must not move model/font/pointer caches into it. Add tests that authored and fallback projection
consume the same committed frames.

## `docs/plans/zircon_runtime/runtime/09`

Coordinate Runtime UI invalidation and retained-surface generations so a tab title, font metrics,
style or available-width change invalidates exactly once. Unchanged input/paint must reuse the same
generation.

## `docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`

Expose or reuse a stable Runtime Text metrics generation in the strip-layout key. Do not add a second
editor-only glyph-width algorithm. Capture title measurement counts around the existing Runtime Text
path.

## Acceptance handoff

Only the plan owner may update the shared ledgers. The handoff must include:

- reviewed fingerprint for 2/2 Rust files;
- managed focused test output;
- baseline and optimized WPR/ETW artifacts outside `C:`;
- layout/title-measure/node-build/allocation counts for the scenario matrix;
- current-source real-window evidence and optional RenderDoc parity capture;
- quantified milestone commit and WeCom notification after acceptance.
