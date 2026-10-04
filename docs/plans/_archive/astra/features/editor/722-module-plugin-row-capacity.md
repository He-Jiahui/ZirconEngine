---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/00-ui-architecture-performance-reassessment-2026-09-02.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/713-viewport-toolbar-cache-remap-ownership.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows.rs
tests:
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows/target_mode_join_tests.rs
---

# Editor module-plugin row projection capacity

The module-plugin pane already receives a concrete plugin list, so its output
row count is known before projection. The row builder now reserves
`report.plugins.len()` and extends the destination with the existing mapper,
removing geometric growth while preserving source order, row fields, and
target-mode string ownership.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor01 / Editor06 / EUI-722 | Reserve the plugin-row projection upper bound before the one-pass mapper. | implemented_pending_validation | RED/GREEN source probes, existing target-mode/source guard, Rustfmt, and scoped diff checks pass; managed Editor Cargo and Release allocation/latency evidence remain pending. |

## Complexity and allocation boundary

Projection remains `O(P)` for `P` plugin rows plus the existing per-row string
fields. The destination vector now allocates once for the known row upper bound;
no cache, plugin authority, ordering, or target-mode join behavior changes.

## Local evidence

- A RED source probe confirmed the explicit capacity path was absent; the GREEN
  probe and existing in-file source guard confirm the reserved mapper path.
- The focused hierarchy/editor contract batch passed `57/57`; the final
  non-tooling Runtime/Editor performance-plus-pressure batch passed `1320/1320`
  across 343 modules in `94.188s`.
- The subsequent Runtime723 output-capacity follow-up reran the same loader and passed
  `1320/1320` in `5.353s`.
- The current combined non-tooling Runtime/Editor batch covers 347 modules and
  passes `1331/1331` tests in `6.361s`.
- Rustfmt, `git diff --check`, and optimization-record whitespace checks pass.
- These checks are source/contract evidence only and do not claim product CPU,
  allocator, RSS, or p50/p95/p99 acceptance.

## Managed acceptance gate

This slice joins the existing asynchronous Runtime/Editor admission recorded in
`696`; no new coordinator request or status query was issued. The row remains
`implemented_pending_validation` until an owner-attributed Windows Release run
measures module-plugin projection allocation and latency at the declared plugin
counts.
