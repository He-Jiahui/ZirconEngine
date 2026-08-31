---
title: Editor workbench reference stable projection index protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-reference-stable-projection-index-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/reference` - 9/9 Rust files source-reviewed; stable indices,
incremental semantic/geometry worksets and full fallback exist, but incremental topology validation
deep-clones identities, geometry patches clone full host nodes, the test-only reference builder is
production-visible, and managed/profile/power acceptance remains pending.

Do not add the folder to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach the M1 allocation removal and M2 compact delta cutover to the retained-host recompute/projection
work under `PERF-MVP-103`, `PERF-MVP-106` and `PERF-MVP-626`. Record K, cloned node count/bytes and
fallback reason; duration alone is insufficient.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of the release-visible handwritten reference builder and any full-node compatibility
patch API after compact deltas are committed. The declarative workbench asset remains the only
production visual truth.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Extend the existing retained-host invalidation audit: topology snapshots are built only on the slow
path; ordinary semantic/geometry transactions carry stable indices and compact changed masks.
Coalesce repeated row changes before one per-frame commit.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Move visual reference parity to declarative component fixtures. Handwritten palette/path/widget
trees must not remain as a second release-visible component implementation.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Pointer/hit consumers rebind from the same generation-checked compact delta. No second scan or full
host node clone may be introduced for hit testing.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the componentized bridge delta publication and presentation commit. Geometry-only resize patches
must carry only row identity plus frame/clip/z; full fallback requires a named generation, index or
topology mismatch.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Runtime UI owns stable surface node identity, changed semantic/geometry masks and committed layout
generation. Editor retained-host projection adapts those receipts; it must not reconstruct Runtime
topology identities on every update.

## Acceptance handoff

The owner handoff requires the 9/9 current-tree fingerprint including the preserved foreign edit,
managed focused tests, K/clone/allocation counters, current-source UI/WPR traces on D/E/F, explicit
fallback coverage, real-window screenshots, power evidence, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.
