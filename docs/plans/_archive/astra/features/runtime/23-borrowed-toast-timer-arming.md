---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/274/2026-09-09-borrowed-toast-timer-arming.md
  - docs/plans/optimize/zircon_runtime/274/2026-08-28-owned-toast-timer-event.md
  - docs/plans/optimize/zircon_runtime/75/2026-08-27-borrowed-toast-queue-scan.md
---

# Borrowed Toast Timer Arming

`UiInputManager` now keeps toast IDs borrowed through queue parsing and surface-state checks. The
timer state owns only the value that must outlive the event, and reuses an unchanged ID while
refreshing its deadline. Existing queue aliases, duration parsing, stale expiry behavior, and the
owned surface accessor remain compatible.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime274/Toast | borrowed queue and surface ID reads; same-ID timer rearm reuse | `implemented_pending_validation` | Focused Rust regression/source contracts, Rustfmt, scoped diff check, release model, and batched Runtime Python `36/36` pass. Managed Cargo, product timer behavior, and release p50/p95/p99 remain pending under the dirty external `E:/Git/zr_vm` checkout. |
