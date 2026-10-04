---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/json_writer.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/json_writer.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-09-27-streaming-profile-json-export.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-27-profile-export-same-session-single-flight.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/streaming_tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export/session_owner_tests.rs
doc_type: milestone-detail
---

# Runtime1018: buffered streaming profile JSON

Status: `source_candidate_pending_managed_validation`.

- [x] Claim five exact paths, save the Runtime1016 predecessor bytes and four
  absent-path preimages, and record the explicit successor relationship.
- [x] Add JSON byte-equivalence and typed failure tests before the writer;
  record a source-only red condition without claiming a Cargo red run.
- [x] Replace per-file full JSON `Vec` serialization with a private buffered
  writer, preserving Perfetto mode and stale-artifact cleanup behavior.
- [x] Add an ignored Release 1K/32K/131K-span paired raw-sample protocol.
- [x] As an exact successor to the frozen streaming source, add process-local
  same-directory single-flight ownership around all Runtime artifact writes,
  with canonical path identity and weak-key cleanup.
- [x] Add test-first same-session alias/Perfetto interleaving, different-session
  concurrency, and error/retry regressions; the test-only owner helper signals
  only after its real `try_lock` returns `WouldBlock`, then obtains the real
  blocking guard that encloses the action. A bounded completion wait and final
  file assertions remain required. Dynamic validation remains pending.
- [ ] Complete independent source review, attribution, and managed Rust
  compilation plus focused tests for the final exact snapshot.
- [ ] Run and evaluate the paired Release samples, allocation/RSS evidence,
  and Runtime03 §9.4 Windows product performance gate.
- [ ] Close remaining P1-7/M6 atomic publication, manifest/hash, cross-process
  and Editor-sidecar ownership, cancellation, and full snapshot/export costs.

Source-only checks and a pending managed request do not establish test or
performance acceptance.
