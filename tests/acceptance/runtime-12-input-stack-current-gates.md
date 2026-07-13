---
related_code:
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/tests/runtime_absorption/input_stack
plan_sources:
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
output_records:
  - docs/plans/zircon_runtime/runtime/12/2026-07-09-input-stack-and-action-mapping-output-records.md
status: owned_filters_accepted_ui_input_failures_remain
---

# Runtime 12 Input Stack Current Gates

Date: 2026-07-11

The input structure audit is `risks = []`: 12/12 runtime modules, 20/20
framework modules, 7/7 test modules, 26/26 public anchors, all five Runtime 12
guards, 15/15 behavior anchors, all documentation and Cargo-gate anchors, and
no oversized owners. `action_map` passes 8/8 and `gamepad` passes 27/27.

The broad `input` text filter is 441 passed / 13 failed / 1 ignored; every
failure is inside the active UI input/text routing workstream. Runtime 12's
owned action/gamepad boundary is accepted, while the cross-owner input gate and
app regression remain open.
