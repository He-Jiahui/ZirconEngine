---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/134-editor-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/context/builder/tests.rs
  - zircon_editor/src/tests/editing/state/play_mode.rs
---

# Editor900 Context and Play-Mode Test Contract Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor context settings startup | Assert that the current mutation coordinator reports `Ready` or `Unavailable` before any settings write, replacing the removed persistence queue accessor without restoring a deprecated public service. | v27 managed Editor library check reported one missing-method error; local Rustfmt and scoped diff checks pass. | implemented_pending_validation |
| Typed Editor event schemas | Compare the existing schema ID's `as_str()` with eight canonical string constants, preserving the full event and payload assertions. | v27 reported eight `EditorMessageSchemaId` versus `str` comparison errors in the clean builder test owner; local source check passes. | implemented_pending_validation |
| Play-mode world/hierarchy regressions | Read node counts through the authoring world's existing callback instead of trying to call `node_records` on `Result<Option<World>>`; use the existing checked reparent method in the two hierarchy assertions. | v27 reported five invalid world-snapshot calls and two removed-method calls in the clean Play Mode tests; local source check passes. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/context/builder/tests.rs` | `A9A9FD473454F71492388EB5B37370C647FC412E2358379B264D7673E81036DD` |
| `zircon_editor/src/tests/editing/state/play_mode.rs` | `9247B14A6F577E45D4032FEACD338F9D8590506CEB26BE8AAFEDAC570E0D1FFF` |

## Managed gate

The terminal v27 Editor test-profile check reached Rust but stopped before
test execution with 220 compiler errors across 110 owner files. These two
clean test repairs postdate that failed snapshot and target 16 of its
diagnostics; they have not been compiled or tested. Group the next managed
Runtime/Editor current-source regression batch after the remaining shared
lower-contract failures are owner-scoped and stable. Neither ignored Release
benchmarks, allocation budgets, nor product p50/p95/p99 are accepted here.
