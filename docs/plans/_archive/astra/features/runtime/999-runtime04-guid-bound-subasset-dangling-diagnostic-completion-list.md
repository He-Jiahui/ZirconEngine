---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-27-guid-bound-subasset-dangling-diagnostic.md
implementation_files:
  - zircon_runtime/src/asset/reference_resolver.rs
tests:
  - zircon_runtime/src/asset/reference_resolver.rs
---

# Runtime04 GUID bound missing subasset diagnostic completion list

| Completed slice | Source evidence | Remaining acceptance |
| --- | --- | --- |
| Bind missing-label diagnostics to the known parent GUID's source. | The resolver checks that source's exact labeled entry and returns `DanglingSubasset` with the persisted hint, label and sorted same-source candidates when absent. It does not consult an obsolete or occupied hint for this decision. | Grouped managed Runtime build and resolver/migration tests must pass on the sealed source. |
| Preserve stable reference identity. | A different exact labeled entry and every mismatched label on an already labeled GUID remain `Conflict`; the missing-GUID branch retains its existing behavior. | Confirm the existing identity-contract regressions in the same managed batch. |
| Cover stale and occupied path hints in the existing resolver fixture. | Tests assert a missing hint, B-occupied hint with and without matching label, same-source exact label rejection, labeled GUID mismatch, and missing-GUID controls. | Run `resolution_keeps_guid_authoritative_and_reports_path_candidates` plus relevant `retired_migration_` tests. |

Validation and performance evidence are pending. This correctness repair does
not close Runtime04 pack/export throughput, mount/read, or memory targets.
