---
title: Editor04 Folder Sort Borrowed Keys
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor04 Folder Sort Borrowed Keys

## Scope and target

The Editor asset catalog rebuild creates a folder tree from the authoritative asset generation. Preserve child-folder display-name order, direct-asset order, parent IDs, and recursive counts while removing temporary copies used only by sorting.

For `F` folders and `A` assets, the target is **zero copied folder-name index entries and zero copied asset display names** during folder sorting. The former path copied each folder ID and display name into a second map (`2F` strings) and copied each asset display name into its UUID lookup map (`A` strings). The paired Windows Release sibling-sort acceptance gate is optimized p95 at most 70% of the former sort's p95. The full large-catalog build must also report p50/p95/p99 for observation. No timing pass is claimed before the managed run.

## Implementation and evidence

- Sibling IDs have a common parent prefix, and their displayed names are the final path segments. The comparator borrows these suffixes directly and retains the ID tie breaker. The temporary folder-name map and its per-comparison hash lookups are removed.
- The asset sorting lookup owns UUID keys but borrows display-name slices from the catalog generation. It no longer copies every display-name string.
- The focused folder-order regression now includes project-root and nested package siblings with Unicode names. Existing folder admission, terminal reuse, and Asset Workspace regressions remain in the batched Editor validation manifest.
- The ignored `editor04_folder_sort_and_large_catalog_release_benchmark` constructs 4,096 sibling folders under an eight-segment project path. It pairs the former copied-name-map sort with the borrowed-name sort in alternating order across 21 samples, checks identical order, and separately times the actual full catalog projection. Its output marker is `PERF_RESULT EDITOR04_FOLDER_SORT_LARGE_CATALOG_BENCH_V1` with p50/p95/p99 nanoseconds for all three measurements.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/folders.rs` | Rustfmt, scoped diff check, and structural target scan passed |
| Cargo | one managed Windows `zircon_editor` package check and focused folder/Asset Workspace tests, coalesced with other Editor repairs | pending coordinator batch |
| Performance | Run ignored `editor04_folder_sort_and_large_catalog_release_benchmark` in the managed Windows Release Editor batch; sort p95 ratio ≤ 0.70 and full catalog p50/p95/p99 reported under `EDITOR04_FOLDER_SORT_LARGE_CATALOG_BENCH_V1` | pending coordinator batch |

This record reports implementation and structural cost reduction only. It becomes accepted after the shared batch reports passing regressions and the declared performance gate.
