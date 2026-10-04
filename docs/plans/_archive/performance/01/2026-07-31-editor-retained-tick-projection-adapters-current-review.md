---
related_code:
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection/mod.rs
tests:
  - direct rustfmt --check 2/3 passed; job_progress.rs has pre-existing test assertion formatting drift
  - current-source Cargo, scale counters, F4 trace, and independent review pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained tick projection adapters current review (2026-07-31)

## Scope

These current-source adapters contain **3/3 Rust files, 332 physical lines, and 7 inline tests**. All three files were externally modified before this review; no Rust content was changed by Performance01.

| Module/file | SHA-256 | Current-source conclusion |
|---|---|---|
| `backend_refresh.rs` | `2E363FF6748D340846BFCBA2F9E7D89ECC85A0CE57CE3B9455F72C98AA147BCB` | Event dequeue count/time slices are already bounded elsewhere. The caller still builds a complete `editor_snapshot()` only to pass `selected_asset_uuid`, but `CatalogChanged` and `ReferenceChanged` already set `refresh_selected_asset_details` unconditionally, so the UUID-dependent branch has no semantic effect. Remove that wide snapshot at the PERF-MVP-104 owner after current-source behavior coverage; keep the existing once-per-nonempty-batch default-scene URI parse. |
| `job_progress.rs` | `5D7F23413FD3CF834C1B3C8F633263F4D084C6F3F96D1F8EFF0B168FF9D86730` | `primary_snapshot()` correctly avoids cloning all active jobs. A stable retained tick still clones the primary label/message, formats task id/detail, and calls the status setter; the setter compares through another owned status snapshot before suppressing invalidation. PERF-MVP-017 must gate projection on the job progress generation, not merely suppress the final paint. |
| `workbench_notifications.rs` | `B9948DEE461C8A6772CB38A64B7FDE4892B98D4EB4602A1B8CD745333D060124` | Pending decision sync first performs the PERF-MVP-105 full-chrome active-template gate. More importantly, `options.is_empty()` returns before calling the bridge, although the bridge contract accepts an empty generation to clear old modal/history state. This contradicts PERF-MVP-596's required empty-generation clear and can leave stale UI while also hiding the only useful no-data transition. General nonempty notification history remains bounded at 64; wider parse/queue work remains PERF-MVP-269. |

## Call and test boundary

The review traced retained tick refresh, asset/resource event flags, primary progress projection and status equality, pending-play decision adaptation, and the notification bridge. Existing tests cover three backend refresh flag combinations, four progress snapshot/format contracts, and two notification bridge behaviors including direct empty-option clearing. There is no app-level regression proving that an empty pending generation reaches the bridge, no counter proving stable progress projection performs zero owned work, and no source guard proving the selected-UUID editor snapshot was removed.

Slint evaluates a property only when its dependency tracker is dirty, and Bevy exposes generation-like change detection through `is_changed`. Zircon should likewise make job progress, backend asset refresh and pending decisions generation-driven before owned UI projection; helper-local caches or a lower global tick rate would retain the wrong authority.

## Dynamic acceptance still open

Run coordinator-managed Cargo plus events 0/1/256/10k and assets 1/1k/100k, recording full editor snapshots, refresh/detail builds, dequeue-to-commit age and UI p95. The selected-UUID-only full snapshot count must be zero. Run 100k stable progress ticks and record primary/current snapshot calls, String clone bytes, formats, setters and invalidations; all must be zero for an unchanged generation. Exercise pending decisions 0/1/128 and histories 0/1/256: a transition to an empty generation must reach the bridge and clear the old modal exactly once, while stable generations perform zero chrome gate/projection work. Preserve event flags/order, terminal progress hiding/cancel semantics, decision capacity/conflict behavior and the 64-row notification history bound. These files remain in `pending.md` until current-source Cargo, counters, F4 behavior and independent review pass.
