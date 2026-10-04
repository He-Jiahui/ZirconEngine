---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/406/2026-09-26-borrowed-physical-import-path.md
tests:
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/tests.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor406 borrowed physical import path

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor406 | Borrow physical paths for duplicate expansion checks and preserve resolution reset | `implemented_pending_validation` | Behavior regression and valid 192-byte multicomponent Windows fixture are written. `EDITOR406_BORROWED_DUPLICATE_PHYSICAL_PATH_BENCH_V1` retains its pending p95 ≤ 65% target; `EDITOR406_PHYSICAL_PATH_UNIQUE_MIXED_BENCH_V1` adds pending all-unique and 50%-unique p95 ≤ 110% non-regression gates. No Release result is claimed before the combined Editor batch. |
