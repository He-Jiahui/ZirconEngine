---
title: Editor57 Virtual Group Single Window Start
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: release_measurement_pending
---

# Editor57 Virtual Group Single Window Start

## Product path and scope

`AssetContentPaintMetadata::visible_node_rows` calls `append_visible_virtual_group_rows` for the current Asset Browser's virtual list and thumbnail paint groups. The previous group loop called `AssetBrowserVirtualization::binding(scroll_px, slot_index)` for every materialized slot. Each binding recomputed the same scroll window start, including viewport arithmetic and logical row limits, then fetched a paint item and searched selected indices even though group visibility only needs the row offset. The window start does not vary by slot within one append call.

The group append now computes the window start once after confirming a nonempty visible damage intersection, derives each slot's logical index and row offset, and checks damage before fetching the paint item. It omits the selected-index search, whose result this function never used. The public single-slot binding and its selected-state behavior remain unchanged. Slot order, wrap behavior, partial-row fallback, item existence, damage clipping, and row append order are preserved. This is a local paint selection improvement; no state is cached across scroll or catalog generations.

The skipped item and selection operations are reads of immutable generations. This append function used neither the fetched item nor the selected flag; it used only `y_offset`. For a slot that survives clipping, the optimized loop still requires `items.get(logical_index)` to succeed before appending, preserving the retired loop's missing-item behavior and visible count. No mutation or callback is skipped.

## Behavior and Release comparison

- `visible_virtual_groups_match_per_slot_binding_across_scroll_and_pool_boundaries` compares the complete appended row sequence and visible-item count against the retired per-slot append loop at the initial window, a nonzero scroll and origin with a partial damage clip, the 100,000-item logical tail page, a 41-slot partial thumbnail row pool, and a single-column list window. It also directly asserts that logical item 99,999 has a visible materialized group at the tail, so shared old/new omissions cannot pass. The 100,000-item fixture uses 256-item logical paint chunks. The regression was authored before the implementation change; managed Cargo execution remains pending.
- Ignored Windows Release `editor57_virtual_group_single_window_release_benchmark` compares the complete `append_visible_virtual_group_rows` operation against the retired loop on the same prebuilt 100,000-item paint generation and nine node rows per materialized group. The 48-group workload represents a 900 by 620 thumbnail viewport with two overscan rows on each side; 240 groups stress five times that slot work. Groups, paint generation, and retained output capacity are built outside the timed region. Each workload performs five warmup batches, then 31 alternating old/new sample pairs. `PERF_RESULT EDITOR57_VIRTUAL_GROUP_SINGLE_WINDOW_BENCH_V1` prints raw samples in acquisition order, p50/p95/p99, OS, architecture, and package version. The managed receipt must supply compiler, build, machine, and cache metadata.
- Pending Release p95 gates are new at most 90% of retired for 48 groups and at most 80% for 240 groups. Both workloads print before either gate asserts. The benchmark measures visible virtual group selection and append work, including slot binding and damage checks. It excludes paint metadata construction, retained rendering, user input, and end-to-end frame timing.

Editor57's 100,000-asset navigation and action p95/p99 targets, as well as its 1,000,000-asset paged-provider target, still require product-level evidence. A helper Release result cannot close those gates. No performance pass is claimed before the combined managed Windows batch produces output.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs` | Rustfmt, scoped diff, UTF-8, LF, and trailing-whitespace checks passed; Cargo pending |
| Behavior | `visible_virtual_groups_match_per_slot_binding_across_scroll_and_pool_boundaries` plus existing virtualization tests | queued for combined Editor Cargo batch |
| Performance | ignored `editor57_virtual_group_single_window_release_benchmark`; p95 ≤ 90% at 48 groups and ≤ 80% at 240 groups | queued for combined Windows Release batch |

This is an implementation candidate pending compilation, behavior regressions, and actual Release measurements.
