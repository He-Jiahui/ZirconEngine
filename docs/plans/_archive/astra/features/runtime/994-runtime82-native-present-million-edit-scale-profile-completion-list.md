---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-26-native-present-million-edit-scale-profile.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files: []
tests:
  - zircon_runtime/tests/runtime82_million_edit_native_present_profile.rs
---

# Runtime82 native present million character edit scale profile completion list

| Plan slice | Completed fixture | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Million-character edit to finished native present and readback sampling | Added an ignored Windows Release integration fixture with five warmups and 31 identical Backspace samples, real input dispatch, retained UI frame submission, live Win32 WGPU present tickets, same-generation pixel checks, raw stage/total times, and before/after process RSS. It uses a ten-second event deadline and a 900-second libtest-thread deadline. | Managed Release diagnostic remains pending. Total time includes GPU finish and readback; RSS includes renderer and capture memory. Frozen product budgets, allocator counts, App/Dynamic Session, monitor scanout, and matched Unreal evidence are absent, so `RTE-GATE-016` and `RTE-GATE-047` remain open. | implemented_pending_validation |

The profile uses a fresh million-character Surface per sample while keeping the
renderer and GPU caches live. Its p50/p95/p99 summaries are descriptive until
the managed run, machine identity, product thresholds, and comparison workload
are reviewed together.
