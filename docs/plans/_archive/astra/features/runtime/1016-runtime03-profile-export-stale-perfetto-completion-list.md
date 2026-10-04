---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
related_docs:
  - docs/crates/zircon_runtime/core/diagnostics/profiling.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-09-27-profile-export-stale-perfetto-cleanup.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
doc_type: milestone-detail
---

# Runtime1016: reused-session Perfetto cleanup

Status: `source_candidate_pending_managed_validation`.

- [x] Capture the source preimage and claim the exact source and record paths.
- [x] Add module-level regressions for fresh no-Perfetto export, same-session
  `true → false` cleanup, and typed failure when removal is obstructed.
- [x] Remove only the stale optional Perfetto artifact when the current export
  excludes it; preserve missing-file success and propagate other I/O errors.
- [x] Align the public export, error, and test contract in `docs/crates/zircon_runtime/core/diagnostics/profiling.md`.
- [ ] Complete independent source review, attribution, and managed Runtime
  compilation/tests for the final source snapshot.
- [ ] Close the remaining Runtime03 P1-7/M6 staging, manifest, hash,
  concurrency, cancellation, and §9.4 performance/product gates.

The source-only test-first check is not a dynamic test result. This record does
not claim that the full artifact pipeline or its performance target is met.
