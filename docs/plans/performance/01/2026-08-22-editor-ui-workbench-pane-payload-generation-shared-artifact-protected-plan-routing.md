---
title: Editor workbench pane payload generation and shared artifact protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-pane-payload-generation-shared-artifact-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders` - 12/12 Rust
files source-reviewed; visible stable panes still rebuild wide owned payloads, Performance Timeline
deep-clones the profile and synchronously re-analyzes all spans, and long tables are not virtualized;
M1/dynamic/profile/power/visual acceptance remains pending.

Do not add the folder to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M1-M5 to `PERF-MVP-103`, `PERF-MVP-106`, `PERF-MVP-113`, `PERF-MVP-143`,
`PERF-MVP-326`, `PERF-MVP-595` and `PERF-MVP-626`. Record per-kind source/projection generation,
payload builds, row/string clone bytes, hotspot task lifecycle, retained/native patches and
main-thread wall. Keep plugin source demand and pane projection as separate counters.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of wide flat compatibility payloads after typed shared artifacts reach all retained and
native consumers. Do not leave a second builder that silently recreates all rows or synchronously
re-analyzes profiling snapshots.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the pane receipt/artifact cache, visible-pane demand index, scoped presentation patch and
retained/native shared ownership. Stable generation refresh, resize, paint and pointer invalidation
must not rebuild unrelated pane payloads.

## `docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md`

Own persistent virtualized hierarchy, inspector, module/plugin, build/export and performance tables,
including selection, expansion, sorting, filtering, controls, empty states and accessibility. Row
materialization must be proportional to visible rows.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Route source and analysis generations as coalesced receipts. A stable or superseded generation does
not enqueue repeated presentation work; tab close and hide cancel optional work within deadline.

## `docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`

Own profile generation, bounded snapshot storage, cancelable hotspot worker, source/analysis
fingerprints and profiling counters. Aggregation must use the shared scheduler and must not execute
under the editor presentation or runtime recorder lock.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed virtualized row/item-source contracts and generation handles used across Runtime UI and
Editor. Host strings are visible-cell products, not the cross-module data authority.

## `docs/plans/zircon_plugins/01-plugin-architecture-core.md`

Own immutable module/plugin catalog generation and shared status artifact. Editor presentation must
not copy a 26-field row per stable refresh, and reload/unload must invalidate exactly one generation.

## `docs/plans/zircon_plugins/09-export-publishing.md`

Own immutable build/export target generation, target identity and progress updates so the visible
pane patches changed targets instead of rebuilding every row.

## Acceptance handoff

The owner handoff requires the 12/12 post-change fingerprint, managed focused tests, the full scale
matrix, per-generation build/clone/row/analysis counters, source-bound WPR and power artifacts on
D/E/F, F0/F4 product interaction, real-window screenshots, RenderDoc visual parity where a GPU path
is involved, a milestone commit and quantified WeCom notification. Shared ledgers remain protected
until then.
