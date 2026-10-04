---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/135-editor-layout-profile-workspace-state-docking-tab-window-restore-schema-migration-current-source-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-19-layout-preset-name-capacity.md
---

# Editor830 · layout preset name capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Merged project/persisted layout-preset name projection | implemented_pending_validation | Asset URI and persisted-map lengths form one pre-reserved upper bound; sort/dedup order and empty behavior remain unchanged. Source/model contract `4/4`; lower order/capacity regression and ignored `EDITOR830_LAYOUT_PRESET_NAME_CAPACITY_BENCH_V1` marker are wired. The current non-tooling batch passes `2239/2239` across `627` modules in `5.505s`, and the nine-slice focused loader passes `33/33` in `0.017s`, with zero failures, errors, or skips. | Managed Cargo/Windows Release, allocator, and layout-preset product p50/p95/p99 evidence remain pending behind the external `E:\Git\zr_vm` dirty-worktree gate. |

## Deterministic model

For 4,096 asset names plus 4,096 persisted names, the zero-capacity result
model changes `12→0` geometric growth events. Sorting, deduplication, and
empty-input zero capacity remain unchanged. This is allocation-shape evidence
only and does not substitute for managed Release timing, allocator, RSS, or
product-percentile evidence.

Tooling production remains out of scope until the planned Rust migration.
