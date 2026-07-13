---
related_code:
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data
plan_sources:
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
output_records:
  - docs/plans/zircon_runtime/runtime/08/2026-07-09-ecs-kernel-data-alignment-output-records.md
status: owned_filters_accepted_entity_external_render_failure
---

# Runtime 08 ECS Kernel Current Gates

Date: 2026-07-11

## Current-source structure evidence

The repository Runtime structure audit completed with `risks = []` for
`ecs_kernel_data_boundary`: 69/69 source owners, 10/10 test owners, 15/15
archetype anchors, 9/9 storage anchors, 9/9 private-re-export anchors, 18/18
component-identity anchors, 10/10 entity lifecycle anchors, 8/8 observer
anchors, 11/11 command anchors, 12/12 event/message anchors, 12/12 resource
identity anchors, 6/6 change-tick anchors, 21/21 test anchors, 16/16 behavior
anchors, 13/13 documentation anchors, and 6/6 Cargo-gate anchors.

## Managed-binary filter evidence

The fresh default-feature Runtime binary produced these focused results:

- `observer`: passed.
- `command`: passed.
- `messages`: passed.
- `change_tick`: passed.
- `ecs`: 340 passed, 0 failed.
- `entity`: 81 passed and 1 failed. The only failure is the Render-owned
  `render_framework_stats_report_neutral_color_lut_readback_identity` test;
  no ECS entity test failed.

## Decision

Runtime 08's owned source boundary and focused ECS filters are accepted. The
historically broad `entity` text filter is not globally green because it also
selects one active Render color-LUT identity test. This record does not claim
the external Render failure is fixed and does not change the plan definition.
