---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-27-streaming-profile-json-export.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner_tests.rs
doc_type: milestone-detail
title: Runtime03 same-session profile export single-flight
date: 2026-09-27
implementation_status: source_candidate
validation_status: managed_validation_pending
performance_status: pending
---

# Runtime03 · same-session export single-flight

## Confirmed writer interleaving

`export_snapshot` writes the native timeline, optional Perfetto timeline, three
hotspot reports, and summary into a deterministic session directory. Two
same-session calls can interleave. If a Perfetto-enabled call pauses after its
native timeline, a Perfetto-disabled call can finish and report no Perfetto
file, then the first call writes that optional file into the same directory.
The Runtime1016 serial `true → false` cleanup does not prevent this overlap.

## Candidate and regression

After creating the export directory, the writer obtains its canonical path
and a process-local keyed owner. It holds that owner through the complete
write/delete/summary/report sequence. An I/O failure releases the owner by
normal scope exit. Registry entries hold weak references and discard expired
keys; its mutex is released before artifact I/O. Canonicalization failure is
reported with `CreateExportDirectory` and the requested directory path.

The new tests pause a real first export after its native file and start a
second export through a `.` alias of the same physical directory. In test
builds, the owner-lock helper first calls `try_lock` on the selected canonical
owner. Only a real `WouldBlock` result consumes the one-shot thread-local hook
and signals the test; the helper then calls `owner.lock()` and returns that
real guard, which remains held through the export action. The hook and probe
are absent from production builds, whose path calls `Mutex::lock` directly.
After the contention signal, a bounded completion wait requires the second
real export to remain incomplete until the first is released. The eventual
no-Perfetto report must agree with the directory. The tests also check that
another session can finish while the first is paused and that a failed write
releases the owner for retry. This source candidate has not had a Cargo red
run or managed test run; dynamic validation remains pending. Test files live
under the managed `CARGO_TARGET_DIR` root.

## Acceptance boundary

This only serializes Runtime's exporters inside one process. It does not
coordinate separate processes, Editor sidecar writers, or readers of the
fixed directory. It does not provide an atomic multi-file generation,
versioned manifest/hash, crash-safe publication, or async cancellation; those
remain open under Runtime03 P1-7/M6. Managed tests and Windows product and
performance evidence are pending.
