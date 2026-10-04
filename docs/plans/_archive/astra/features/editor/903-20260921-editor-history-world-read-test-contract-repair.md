---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/tests/editing/history.rs
---

# Editor903 History World-Read Test Contract Repair

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Undo/redo and hierarchy reuse tests | Read the existing authoring world's node counts directly through `expect_with_world` in five test locations, retaining creation, undo, redo, and thousand-node reuse assertions. | v27 Editor check reported five `node_records` calls on `Result<Option<World>>` in this clean test owner. Local Rustfmt and scoped diff checks pass; no product world API changed. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/tests/editing/history.rs` | `13C07A9E16DB34E348B0315AD26D99AE2E8332C674E015F6C8FDB2F4F8C6FDD3` |

## Managed gate

These five test repairs postdate terminal v27. No managed Editor test has
executed after them; batch the next Runtime/Editor test-profile validation
only after the remaining shared lower owners stabilize. Release performance,
allocator budgets, and product p50/p95/p99 remain unaccepted.
