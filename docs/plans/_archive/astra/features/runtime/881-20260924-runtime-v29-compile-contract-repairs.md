---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/206/2026-09-15-incremental-publication-capacity.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-notification-static-key-update.md
  - docs/plans/optimize/zircon_runtime/589/2026-09-20-v2-file-source-capacity.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/editor/922-20260924-editor-v29-compile-contract-repairs.md
---

# Runtime881 v29 Lower-Layer Compile Contract Repairs

## 计划完成列表

| Scope | Completed change | Evidence and remaining gate |
| --- | --- | --- |
| Asset incremental publication | Keep growth-event totals in the benchmark's outer scope; measure samples as `Duration` instead of narrowing nanoseconds from `u128` to `u64`. | Six v29 compiler diagnostics targeted; unchanged growth-event assertions and paired p95 marker; managed Release gate pending. |
| Notification and V2 file cache | Resolve their already-present sibling test files through explicit module paths; preserve existing production capacity behavior. | Two v29 missing-module errors targeted; library tests pending. |
| Session, UI input, text and fonts | Correct test imports to existing owner modules; expose focused-surface update only to the sibling Runtime UI module; move thread receivers into closures; use a scoped test-only font directory without a new Cargo dependency. | v29 unresolved symbols, privacy, capture and undeclared-dependency errors targeted; no assertions removed. |
| Rendering and navigation fixtures | Import already-defined render-scene, GPU math and upload types; repair `splitn` and string-range search signatures; isolate anti-alias settings before mutable fixture borrows. | v29 source diagnostics targeted while production render pipeline and fixture behavior stay unchanged. |
| Other lower regressions | Repair nested scope imports, owned Arc comparisons, TOML map creation, the empty mirror assertion and job-task inventory re-export. | v29 compile diagnostics targeted; current-source Rust confirmation pending. |

These changes are source-level repairs to the v29 sealed diagnostics, **not** a
passing compile, test suite, Release benchmark, allocator check or product
p50/p95/p99 measurement. All affected files are shared-checkout edits; foreign
changes outside the minimal repaired expressions were preserved. The next
Runtime/Editor validation is grouped and asynchronous. Tooling is deferred.

Local Rustfmt `--check` passed for the 22 touched Runtime files; the record's
five plan/related links resolve. The only coordinator admission check found
two preexisting running Cargo blockers, so no later compiler result covers this
source and no direct validation launcher was submitted.
The four-module Runtime206, notification, V2-file and Editor asset-index
Python source-contract batch passed `19/19` in one process; it does not compile
Rust or establish a percentile result.
The subsequent repository-wide `test_*performance_contract.py` discovery ran
`2950/2950` Python tests in one process (`40.453s`). Its discovery also includes
tooling test modules; the emitted tooling timing markers are unrelated to this
Runtime completion gate and are not cited as Runtime product timings.
The later grouped Runtime `test_runtime*contract.py` source-contract discovery
passed `1931/1931` in one process. This adds source-shape coverage only; the
current-source managed Rust library, ignored Release benchmarks, allocations,
and product percentile gates remain open while an unrelated Cargo owner holds
the validation lane.
After the grouped v30 Runtime/Editor managed library check-and-test handoff
(`2026-09-24T20:43:59+08:00`, launcher PID `26456`), a combined Runtime/Editor
pressure-contract batch passed `264/264`. An additional 81-module functional /
structure batch ran `232` tests in one process: `231` passed and one Runtime
navigation structure assertion failed. The new
`navigation/runtime/world_scan/capacity_tests.rs` regression raises the current
Rust file count to `17`, whereas the existing skill audit
and `tools/tests/test_runtime_module_family_boundary.py` still require `16`.
No Rust module was removed and no skill/tooling implementation was changed;
that source-inventory test remains RED under the deferred tooling boundary.
Neither Python batch is evidence that v30 compiled, ran Rust tests, or passed
the Release performance gates.
The v30 Runtime package subsequently ended before Cargo check with managed
`compile_input_changed` for a concurrent foreign edit in
`zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs`.
Therefore v30 cannot validate the Runtime881 source repair, and no per-file
Cargo retry was launched while the grouped Editor package continued.
