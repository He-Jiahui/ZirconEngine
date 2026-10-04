---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
related_docs:
  - docs/crates/zircon_runtime/core/diagnostics/profiling.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
doc_type: milestone-detail
title: Runtime03 reused-session optional Perfetto artifact cleanup
date: 2026-09-27
implementation_status: source_candidate
validation_status: managed_validation_pending
performance_status: pending
---

# Runtime03 · reused-session Perfetto cleanup

## Confirmed gap

`export_snapshot` reuses a deterministic directory for each session. Exporting
that session once with Perfetto and then without it left the previous
`timeline.perfetto.json` on disk, although the second report omitted it from
`files`. A consumer scanning the directory could mistake the old trace for the
new capture. The existing no-Perfetto test only used a fresh directory.

## Candidate and regression

- In the no-Perfetto branch, remove only that session directory's optional
  Perfetto file. Missing files are valid; any other removal error returns a
  typed `ProfileExportError::RemoveFile` instead of a successful report.
- The public export, error, and test contract is synchronized in `docs/crates/zircon_runtime/core/diagnostics/profiling.md`.
- The module tests now cover fresh export without Perfetto, `true → false`
  export of the same session with the same directory, and an obstructed stale
  file removal. The two new behavior tests were written before the production
  branch; their source-only red check established that the old branch had no
  removal call. No Cargo red run was made.

## Acceptance boundary

This candidate closes only the stale optional artifact subcase of P1-7/M6.
Atomic staging and publish, manifest/hash/fingerprint consistency, concurrent
same-session exports, and async cancellation remain open. Managed compilation,
the relevant Runtime tests, and Runtime03 §9.4 performance/product evidence are
pending; source checks alone do not establish those gates.
