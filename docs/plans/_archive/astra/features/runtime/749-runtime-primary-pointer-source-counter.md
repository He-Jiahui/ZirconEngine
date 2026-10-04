---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-14-primary-pointer-source-counter.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
related_code:
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table/index_tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
tests:
  - tools/tests/test_runtime_ui_primary_pointer_source_counter_performance_contract.py
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs (lifecycle regression)
---

# Runtime749 · Primary pointer source counter

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime200 / primary pointer admission | Keep Touch/Pen primary membership in table-owned counters and replace the fallback full scan with an expected-constant-time probe. | implemented_pending_validation | Source/lifecycle/pressure contracts pass `15/15`; the Rust lifecycle regression compares counters with the legacy scan across multiple-primary transitions; one-process all-current Runtime/Editor batch covers `870` modules and passes `3562/3562`; scoped Rustfmt and Python compilation pass; Release marker and managed Cargo, allocation, behavior, and pointer-input p50/p95/p99 evidence remain pending. |

## 证据

| 项目 | 状态 |
| --- | --- |
| focused source contracts | `15/15` passed in one batch |
| all-current Runtime/Editor contract batch | `870` modules / `3562/3562` tests passed in `517.932s` (latest one-process confirmation; prior receipt `396.106s`) |
| scoped rustfmt | passed (`pointer_table.rs`, `index_tests.rs`, `manager.rs`) |
| deterministic model | `100,000` legacy scans → `1` counter probe per admission; Release marker reports `4096` legacy membership steps vs `1` counter step per probe and places the primary entry at the tail |
| managed validation | pending under the shared external-worktree admission blocker |

实现细节与边界见 [Runtime200 primary pointer source counter](../../../../plans/optimize/zircon_runtime/200/2026-09-14-primary-pointer-source-counter.md)。
