---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/23/2026-08-26-binding-payload-borrowed-root.md
  - docs/plans/optimize/zircon_editor/23/2026-08-26-preview-suggestion-borrowed-root.md
  - docs/plans/optimize/zircon_editor/23/2026-09-19-payload-suggestions-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/asset_editor/binding/payload_suggestions.rs
  - zircon_editor/src/ui/asset_editor/binding/payload_suggestions/borrowed_root_tests.rs
tests:
  - tools/tests/test_editor_payload_suggestions_capacity_performance_contract.py
---

# Editor836 · payload suggestions capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor23 binding payload suggestions | Reserve array/template and table-key output bounds before direct extension while preserving borrowed-root lookup, duplicate last-wins behavior, append index, sorted table order, and owned returned values. | TDD source/model contract `3/3`; lower Rust source regression and ignored `EDITOR836_PAYLOAD_SUGGESTIONS_CAPACITY_BENCH_V1` marker are wired; deterministic dense model changes `12→0` array and `11→0` table growth events; focused batch `635/635` across `172` files and broad non-tooling contract batch `3352/3352` across `826` files both pass. Managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only temporary vector capacity in the immediate nested
suggestion projection. It does not alter path parsing, root borrowing,
duplicate-key precedence, append-index selection, table sorting, value cloning,
or empty/scalar fallback behavior.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/asset_editor/binding/payload_suggestions.rs` | `C9EA7A25EECA5FACA4F8BBBC6BF0E8E639611EF8FEA8F7F358C6384E10C215E4` |
| `zircon_editor/src/ui/asset_editor/binding/payload_suggestions/borrowed_root_tests.rs` | `4492EA56C058AC5C69759973A4E74486897BB1C7D63B2EE7F24395A1C7517CEF` |
| `tools/tests/test_editor_payload_suggestions_capacity_performance_contract.py` | `D579B475363C84591F13D6FF2DA50A3ED888A850872365EE9B21A3C9F95B5834` |

## Managed gate

No Cargo process is started locally and the coordinator is not polled. Keep
this entry `implemented_pending_validation` until the owner-attributed Windows
Release lane proves current-source compilation, lower regressions, output
parity, allocation behavior, and Editor payload p50/p95/p99 evidence.
