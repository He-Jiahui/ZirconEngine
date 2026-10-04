---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/json_writer.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/json_writer.rs
related_docs:
  - docs/crates/zircon_runtime/core/diagnostics/profiling.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-27-profile-export-stale-perfetto-cleanup.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/streaming_tests.rs
doc_type: milestone-detail
title: Runtime03 buffered streaming JSON profile export
date: 2026-09-27
implementation_status: source_candidate
validation_status: managed_validation_pending
performance_status: release_raw_samples_pending
---

# Runtime03 · buffered streaming JSON export

## Confirmed cost and candidate

Each JSON artifact in `export_snapshot` previously used
`serde_json::to_vec_pretty` followed by `fs::write`. A large native timeline or
Perfetto trace therefore allocated an entire serialized file in addition to
the in-memory snapshot/projection. The export now writes the same pretty JSON
representation through a 64 KiB `BufWriter<File>` and explicitly flushes it.
Native timeline, optional Perfetto timeline, and three hotspot reports use the
same private writer. The Runtime1016 `true → false` stale Perfetto removal is
preserved.

`serde_json` wraps underlying writer errors. The writer maps its I/O category
back to `ProfileExportError::WriteFile`, carrying the artifact path and error
kind; non-I/O serialization errors remain `JsonSerialize`. File creation and
final flush errors are also typed `WriteFile`.

## Regression and measurement protocol

- New focused tests compare every exported JSON file byte for byte with the
  previous `to_vec_pretty` representation for both Perfetto modes, including
  escaped text. They check typed create, mid-stream write, flush, and
  serialization failures. These tests were authored before the writer and a
  source-only red condition was recorded; no dynamic red run occurred.
- An ignored Release evidence test prepares 1,024, 32,768, and 131,072 real
  spans outside timing. For each native/Perfetto mode it performs five warmups
  and 31 alternating legacy/streaming pairs, timing serialization plus file
  write and comparing the resulting bytes outside timing. It emits every raw
  nanosecond pair plus nearest-rank P50/P95/P99. The old in-memory
  serialization remains only in tests as the frozen comparison method.
- Managed Windows Release execution, raw sample review, allocations/RSS,
  source fingerprint, host specification, and product export behavior are
  pending. No measured performance improvement or Runtime03 §9.4 gate is
  claimed from source inspection.

## Remaining boundary

This candidate removes the full serialized JSON `Vec` for these files only.
`ProfileExportReport.snapshot.clone()`, the synchronous session/export owner,
and `summary.md` formatting remain. P1-7/M6 atomic staging and publication,
generation manifest/hash, concurrent same-session owner, and async
cancellation remain open. A serialization or I/O failure can still leave
a partial file in the reused session directory; the existing export API
reports failure instead of a complete `files` manifest.
