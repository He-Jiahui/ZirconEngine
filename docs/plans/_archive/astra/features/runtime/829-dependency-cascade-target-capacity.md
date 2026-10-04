---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-08-26-dependency-cascade-hash-visited.md
  - docs/plans/optimize/zircon_runtime/74/2026-09-19-dependency-cascade-target-capacity.md
---

# Runtime829 · dependency cascade target capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Reverse-dependency cascade target collector | implemented_pending_validation | First admitted dependent lazily reserves the current fanout; BFS order, borrowed hash visitation, duplicate suppression, empty lookup, and self-cycle behavior remain unchanged. Source/model contract `4/4`; lower fanout/empty/cycle regression and ignored `RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1` marker are wired. The current one-process non-tooling batch loads `627` modules and passes `2239/2239` in `5.505s`; the nine-slice focused loader passes `33/33` in `0.017s`, all with zero failures, errors, or skips. | Managed Cargo/Windows Release, allocator, and dependency-cascade product p50/p95/p99 evidence remain pending behind the external `E:\Git\zr_vm` dirty-worktree gate. |

## Deterministic model

For a 4,096-target first wave, the zero-capacity vector model changes `11` geometric growth
events to `0`; an empty or self-cycle traversal retains zero target capacity. This is
allocation-shape evidence only and does not substitute for managed Release timing, allocator,
RSS, or product-percentile evidence.

Tooling production remains out of scope until the planned Rust migration.
