---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/636/2026-09-01-indexed-shell-instance-repair.md
related_code:
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout.rs
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout/optimization_batch_iy_editor636_tests.rs
tests:
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout/optimization_batch_iy_editor636_tests.rs
---

# Indexed Shell Instance Repair

Built-in shell repair now builds borrowed indexes once per repair for open instance and descriptor
IDs, replacing repeated linear lookups while preserving exact-ID priority, first descriptor match,
ordering, and drawer visibility semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor636 | Reuse bounded open-instance indexes during shell repair | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
