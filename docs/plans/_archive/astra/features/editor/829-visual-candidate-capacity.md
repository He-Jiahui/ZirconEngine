---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/123/2026-08-26-visual-asset-candidate-last-hit.md
  - docs/plans/optimize/zircon_editor/01/2026-09-19-visual-candidate-capacity.md
---

# Editor829 · visual candidate capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Packaged image/preview/icon candidate capacity | implemented_pending_validation | Nonempty image, relative preview, and packaged icon collectors reserve finite variant bounds; absolute preview sources use one entry; empty paths remain zero-capacity and development module candidates remain extensible. Source/model contract `4/4`; lower cardinality/order/bound regression and ignored `EDITOR829_VISUAL_CANDIDATE_CAPACITY_BENCH_V1` marker are wired. The current non-tooling batch passes `2239/2239` across `627` modules in `5.505s` and the nine-slice focused loader passes `33/33` in `0.017s`. | Managed Cargo/Windows Release, allocator, and visual-resource product p50/p95/p99 evidence remain pending behind the external `E:\Git\zr_vm` dirty-worktree gate. |

## Deterministic model

For representative 4/5/6 candidate variant counts, the zero-capacity model
removes five geometric growth events across the three collectors (`5→0`). Empty image input
retains zero capacity. This is allocation-shape evidence only and does not
substitute for managed Release timing, allocator, RSS, or product-percentile
evidence.

Tooling production remains out of scope until the planned Rust migration.
