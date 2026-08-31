---
related_code:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging/tests.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging/tests.rs
  - scoped source ownership scan
  - scoped production-file budget mirror scan
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 IBL source cubemap staging test owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | IBL source cubemap staging folder-backed test owner split | `runtime_15_ibl_source_cubemap_staging_test_owner_split_source_complete_static_passed_managed_validation_deferred` | 2026-08-27 | Mixed owner 1414 lines -> production owner 776 lines plus test owner 638 lines; 15/15 tests retained. |

Completed:

- Replaced the inline `tests` module in `ibl_source_cubemap_staging.rs` with an explicit folder-backed path mount.
- Moved all 15 source path, two-phase publication, recovery, snapshot retry, and bundle barrier tests into `ibl_source_cubemap_staging/tests.rs`.
- Preserved the current uncommitted prepared-write and recovery coverage while leaving bundle publication behavior and production APIs unchanged.
- Removed this owner from the Runtime 15 production-file over-budget set: the production owner is now 776 lines, below the 800-line limit.

## Review basis

This is a mechanical ownership correction required by `engine-code-structure-convention.md`. The IBL staging store, durable transaction policy, validation, decoding, and request identity logic remain in the production feature owner; only feature-local tests moved. No compatibility module, alias, re-export, duplicate implementation, algorithm replacement, or hotpath instrumentation was introduced.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for both Rust owners.
- Static ownership scan reported root test attributes `0`, child test attributes `15`, explicit child mounts `1`, and retained child `super::` references `3`.
- The production-file budget mirror scan no longer reports `asset/artifact/ibl_source_cubemap_staging.rs` or `core/framework/render/shader/variant_miss_report.rs`; it still reports 50 unrelated current production files at or above 800 lines, so the global Runtime 15 budget remains open.
- Scoped `git diff --check` passed apart from the repository checkout's LF/CRLF notices.
- Managed Cargo validation was not run while bypassing the current validation blocker. Compile and focused behavior tests remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the staging algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for IBL source cubemap staging tests. Managed compile/test, the remaining production-file budget, milestone commit, coordinator integration receipt, and WeCom publication remain open.
