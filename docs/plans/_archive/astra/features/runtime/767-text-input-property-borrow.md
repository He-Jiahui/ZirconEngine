---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-text-input-property-borrow.md
related_records:
  - docs/plans/astra/features/runtime/766-keyboard-indexed-entry-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/text_input.rs
  - zircon_runtime/src/ui/component/state_reducer/text_input/property_borrow_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/text_input/property_borrow_tests.rs
  - tools/tests/test_runtime_text_input_property_borrow_performance_contract.py
---

# Runtime767 · TextInput property borrow

TextInput validation now chooses from static ordered property slices and borrows the selected
textual value during validation. Mirror target slices are also static. Candidate priority,
required/min/max validation, and value/value-text mirroring remain compatible with the previous
behavior.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / TextInput validation and mirroring | Replace transient candidate/mirror vectors and validation-text clone with static slices and borrowed textual values. | TDD source contract GREEN `4/4`; lower candidate/mirror-order regression and ignored `RUNTIME767_TEXT_INPUT_PROPERTY_BORROW_BENCH_V1` marker; combined Runtime/Editor contract batch passes `69/69` in `0.577s`. | implemented_pending_validation |

## 性能边界

Per validation event, the finite candidate container and selected-text clone are removed; mirror
target allocation is removed as well. Local source/model evidence does not establish allocator,
CPU/RSS, or product input p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
