---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_disabled.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_disabled.rs
plan_sources:
  - user: 2026-07-10 frameworks foundation architecture hard cutover
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_enabled.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - tests/acceptance/frameworks-03-physics-contract-feature-boundary.md
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
doc_type: module-detail
---

# Runtime Diagnostics

## Purpose

Runtime diagnostics exposes read-only, tooling-facing snapshots without making Editor or core diagnostic DTOs depend on optional subsystem contracts. Each optional domain is projected into an always-available diagnostic representation at the collection boundary.

## Physics Projection

`RuntimePhysicsBackendDiagnostics` is the neutral projection of the optional Physics backend status. It preserves requested and active backend names, state, detail, simulation mode, and feature-gate information using always-on standard-library field types. It deliberately does not re-export or embed `PhysicsBackendStatus`, `PhysicsBackendState`, or `PhysicsSimulationMode`.

The feature-enabled collector resolves `PhysicsManager`, reads its settings and backend status, and converts contract enums to stable snake-case diagnostic strings. The feature-disabled collector reports that Physics contracts were not compiled. `collect.rs` therefore needs only to invoke the selected collector and never imports or resolves the optional Physics manager itself.

This split keeps diagnostic consumers source-compatible across Runtime profiles without pretending that an unavailable Physics contract is a live backend. The Editor renders the neutral fields and does not import Physics framework types.

## Current Integration State

The neutral DTO and enabled/disabled collectors are staged but not yet mounted from `diagnostics/mod.rs`; existing production collection remains active while shared Runtime builds are running. This hard cut is incomplete until the declaration adapters are selected, `physics.rs` uses the neutral DTO, `collect.rs` delegates collection, Editor fixtures and presentation code use the neutral fields, and all old optional-contract imports are removed in the same public cutover.

No compatibility alias from `PhysicsBackendStatus` to the neutral DTO is permitted.

## Validation

The Frameworks 03 static boundary test requires both declaration-selected collectors, rejects `resolve_physics_manager` in the common collector, rejects optional Physics imports in the neutral DTO, and requires the neutral backend projection in `RuntimePhysicsDiagnostics`. Profile checks must then prove Client and Editor collect Physics when enabled while Server and `core-min` compile the unavailable projection without Physics declarations.
