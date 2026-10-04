---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/77/2026-08-26-input-timer-retain-drain.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-19-input-timer-drain-capacity.md
---

# Runtime824 · Input timer drain capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Four UI input timer drains lazily reserve expiration-map bounds | implemented_pending_validation | RED→GREEN source contract `3/3`; lower order/pending/capacity regression and ignored `RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1` marker wired; no-expiry drain remains zero-capacity | Managed Cargo/Windows Release and input product p50/p95/p99 remain pending behind external `E:\Git\zr_vm` dirt |

Tooling production remains out of scope until the planned Rust migration.
