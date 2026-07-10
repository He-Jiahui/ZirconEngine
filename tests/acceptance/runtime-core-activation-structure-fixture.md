---
related_code:
  - zircon_runtime/src/core/runtime/tests/activation/structure/fixture.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_dependencies.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/blocked_unload.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
output_records:
  - docs/plans/zircon_runtime/runtime/02/2026-07-09-core-spine-and-root-surface-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
---

# Runtime Core Activation Structure Fixture Acceptance

Date: 2026-07-10

The Runtime 15 blocked-deactivation test family is folder-backed. Its shared structure fixture now reads the exact-two/three dependency matcher, exact-four dependency matcher, and exact-five no-index-map children instead of expecting their tests in the retired parent.

Verification:

- Default-feature direct `core::` filter: 641 tests, 629 passed, 12 failed, 6797 filtered out, 51.57s.
- The two owned activation structure failures were reproduced in that run.
- Standalone current-source activation structure harness: 2 passed, 0 failed for the affected guards.

Status `runtime_02_core_activation_structure_fixture_inventory_static_passed_core_gate_external_failures_remain` does not close Runtime 02. Ten remaining failures are in active Render/UI owners, so the complete `core::` Cargo gate remains pending.
