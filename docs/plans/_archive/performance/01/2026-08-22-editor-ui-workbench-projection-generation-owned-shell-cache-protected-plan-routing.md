---
title: Editor workbench projection generation-owned shell cache protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-projection-generation-owned-shell-cache-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/{projection_cache.rs,shell_presentation.rs}`
- 2/2 Rust files source-reviewed; upstream rebuilds the full workbench model before downstream caches
perform O(source/menu-tree) deep comparisons, menu hits probe the global template store, and shell
segments lack exact generations; M1 removed the fresh-cache wrapper chain, while M0/M2-M6
dynamic/profile/power/visual acceptance remains pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to `PERF-MVP-106`, `PERF-MVP-113`, `PERF-MVP-147` and `PERF-MVP-626`. Record exact model
segment generations, equality items/bytes, source/output clone bytes, template-store mutex/load/path
work, scoped/full applies, main-thread CPU, allocations, latency, RSS and package energy.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of fresh-cache constructors, deep-value source snapshots and full-model compatibility
routes after every consumer uses committed immutable receipts. Do not retain hash/equality shims as a
second generation authority.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `WorkbenchProjectionReceipt`, immutable model/presentation segments and one authoritative shell
assembly. Stable full refresh must be O(1) and narrow invalidation must replace one segment.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own committed receipt consumption by drag/drop, pointer and hit-test routes. Input queries must not
build ad hoc complete workbench models.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Publish and coalesce exact menu/tab/drawer/floating/pane/status generations once per frame. Preserve
the dirty domain through model, presentation, scene and native apply; no silent Full escalation.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Own immutable shared segment types and typed receipt comparison. Value-tree hashing or recursive
equality is not an accepted substitute for source ownership.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own a lock-light template/resource/font generation snapshot updated by the store owner. Generation
reads must not enter the global load mutex or allocate request/path structures.

## Acceptance handoff

The owner handoff requires 2/2 post-change fingerprints, managed focused and behavior tests, the full
scale matrix, current-source WPR/power artifacts on D/E/F, interaction and screenshot checks,
RenderDoc parity for GPU content, milestone commit and quantified WeCom notification. Shared ledgers
remain protected until then.
