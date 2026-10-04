---
title: Editor asset content virtualized item source protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-asset-content-virtualized-item-source-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/asset_content_layout` - 8/8 Rust files source-reviewed; O(1)
thumbnail hit and visible-group partitioning are retained, but full-catalog DTO generation and
Activity/list/thumbnail layout remain Theta(A squared). Typed item-source virtualization, managed
tests and current-source WPR/allocation/power evidence are pending.

Do not add the folder to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Promote the existing asset-content work under `PERF-MVP-219` from metadata cleanup to MVP-P0
virtualization. Baseline and acceptance must include generation/layout work before paint, not only
the existing 10k metadata visible-row fixture. Keep `PERF-MVP-219` open until the Editor09 failure is
returned with dynamic evidence.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of full-catalog dynamic template nodes, duplicate control-ID parsers/string identity
maps and the second filename compaction policy after all consumers use typed visible batches. No
compatibility renderer or fallback parser may survive.

## `docs/plans/zircon_editor/editor/09-editor-asset-management.md`

Extend `failure-2026-07-17-asset-pane-projector-repeated-model-scans.md`. The previous painter-side
O(visible) fix is retained, but the failure remains open because upstream Activity/list/thumbnail
generation and layout are Theta(A squared), and paint still allocates/sorts/parses visible routes.
Editor09 owns stable typed asset item sources, filtering/sorting generations and preview readiness.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Coalesce asset catalog, search, filter, selection and preview-ready invalidations. Expensive filter
work must be cancellable and time-sliced; resize/scroll may update virtual layout without rebuilding
the catalog source.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Define one virtual list/tree/reference/thumbnail component contract with typed item identity,
visible/overscan range, exact extent and reusable presentation slots. Generic control-ID strings are
not a row-layout or route index.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Pointer and hover routing consume typed visible slot handles from the committed generation. Remove
per-node source/reference prefix parsing and `BTreeMap<String, identity>` lookup.

## `docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`

Own the editor-facing virtual asset surfaces and bounded preview prefetch policy. List, tile, folder
tree and reference panels share one item-source generation but independent viewport state.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Publish the committed virtual layout/visible batch to projection, paint, scrollbar and pointer
consumers. Stable redraw and unrelated Host recompute must perform zero asset projection/layout work.

## `docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`

Publish compact revisioned catalog/query data and cancellable preview readiness without editor DTOs.
Do not move Editor layout or widget ownership into Runtime asset management.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Provide the generic typed virtual collection primitive, invalidation receipt and reusable visible
slot lifecycle. The editor supplies asset-specific data and presentation; Runtime UI owns the
collection algorithm and O(log A + V) contract.

## Acceptance handoff

The owner handoff requires the 8/8 fingerprint, corrected static contracts, managed focused Rust
tests, 1/100/1k/10k/100k scale counters, current-source WPR/ETW and allocation artifacts on D/E/F,
real-window interaction and pixel parity, before/after power data, milestone commit and quantified
WeCom notification. Shared ledgers remain protected until then.
