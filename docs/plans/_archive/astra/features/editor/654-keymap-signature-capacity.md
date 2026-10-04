---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/654/2026-09-01-preallocated-keymap-signature-buckets.md
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/optimization_batch_jo_editor654_tests.rs
tests:
  - zircon_editor/src/core/commands/keymap/optimization_batch_jo_editor654_tests.rs
---

# Keymap Signature Index Capacity

Editor keymap construction now reserves its signature index from the effective binding count.
Signature ordering, collision behavior, keyboard resolution, and override semantics remain
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor654 | Bound keymap signature-bucket capacity before binding insertion | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
