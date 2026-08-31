---
title: Workbench page tabs single strip layout authority protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-page-tabs-single-strip-layout-authority-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/page_tabs` - 2/2 Rust files reviewed and static-format passed; pure
metrics are bounded constant-time, while one committed projection/pointer/overflow layout authority,
managed tests and current-source WPR/allocation/power evidence remain pending.

Do not add the folder to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach this work to `PERF-MVP-106`. Stable unrelated Host recompute requires zero page clones, Runtime
Text measures, hidden-index allocation, pointer layout builds and compare-after-build discards.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

After all consumers use the committed page-strip artifact, remove duplicate production visible-index
and geometry allocators. Do not retain a pointer-only compatibility allocator.

## `docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md`

Keep the current responsive/overflow policy and metrics helper. Add the immutable strip-layout owner,
generation key and 320/640/900/1260/1920 plus scale matrix. Projection, pointer and overflow menu must
share exact frames and hidden identities.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Consume stable page/close/overflow route handles from the committed strip generation. No pointer sync
may clone page titles or create a second visibility set.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own page-strip generation publication and the removal of compare-after-build pointer synchronization.
The Host recompute generation DAG skips the entire page-strip consumer when inputs are unchanged.

## Autolayout plan

Route to the current autolayout single-authority plan: publish the resolved `WorkbenchLayoutTier` in
the committed runtime layout generation. Page tabs consume the receipt and do not independently
classify width as a second layout authority.

## Acceptance handoff

The owner handoff requires the 2/2 source fingerprint, managed focused tests, scale/counter matrix,
baseline and optimized WPR/ETW artifacts on D/E/F, current-source screenshots/interaction parity,
milestone commit and quantified WeCom notification. Shared ledgers remain protected until then.
