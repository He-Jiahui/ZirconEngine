---
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/physics/mod.rs
  - zircon_runtime/src/core/framework/scene/physics/combine_rule.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_serde.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/scene/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/scene/physics/tests.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/scene/physics/mod.rs
  - zircon_runtime/src/core/framework/scene/physics/combine_rule.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_serde.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/scene/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/skeleton_joint_binding.rs
plan_sources:
  - user: 2026-07-10 frameworks foundation architecture hard cutover
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/core/framework/scene/physics/tests.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - tests/acceptance/frameworks-03-physics-contract-feature-boundary.md
doc_type: module-detail
---

# Scene Physics Schema

## Purpose

`zircon_runtime::core::framework::scene::physics` is the always-on owner for physics data that is authored and persisted as part of a scene or asset. It exists independently of a compiled simulation backend so a headless asset tool, server build, editor serializer, or project loader can read scene data without enabling `physics-contracts`.

This module owns only authored schema:

- material coefficients and combine rules;
- per-axis joint limits and drives;
- break and projection tolerances;
- optional skeleton and bone bindings.

Backend status, simulation settings, query packets, world-sync packets, events, and the `PhysicsManager` service remain in the optional `core::framework::physics` contract domain.

## Current Integration State

The hard cut is mounted and complete. `core::framework::scene::mod.rs` always publishes `scene::physics`; scene assets, ECS components, project IO, property conversion, framework sync DTOs, Runtime tests, Editor consumers, and Physics plugin code import this owner directly. The superseded `combine_rule.rs`, `joint_constraint_metadata.rs`, `joint_drive.rs`, `material_metadata.rs`, and `skeleton_joint_binding.rs` declarations under the optional Physics contract root are deleted.

There is no compatibility re-export, alias, or duplicate schema under `core::framework::physics`. The optional simulation contract depends on this always-on scene schema in one direction.

## Module Shape

`mod.rs` is structural only. Each public declaration has one file. Custom joint-constraint serialization lives in `joint_constraint_serde.rs` instead of the declaration file, keeping the declaration limited to fields, derives, serde policy, and its default invariant.

`PhysicsJointConstraintMetadata` serializes sparse axis limits as `x`, `y`, and `z` map entries. Deserialization also accepts a three-slot sequence because both are current serialized input shapes; arrays longer than three axes, duplicate map keys, and unknown keys are rejected rather than truncated.

Default drives and absent optional limits are omitted from human-readable output. This keeps scene TOML compact without moving serialization behavior into asset or plugin code.

## Ownership And Data Flow

Asset schemas and ECS components store these types directly. When `physics-contracts` is compiled, the optional world-sync DTOs reference the same persisted types and the Physics plugin projects them into its backend representation. The simulation layer may validate or reject authored values, but it does not become their persistence owner.

This follows the repository-local reference split:

- Godot keeps `scene/resources/physics_material` separate from `servers/physics_3d` implementations.
- Fyrox keeps scene collider and joint data separate from `scene/graph/physics` simulation synchronization.
- Unreal PhysicsCore keeps physical-material authored properties in a dedicated material type rather than a backend manager surface.

Zircon places the schema inside the existing framework scene contract instead of creating a fourth root package.

## Edge Cases And Constraints

- Scene and asset loading must remain available without `physics-contracts`.
- Persisted values may be non-finite or physically invalid at decode time; backend validation remains responsible for refusing invalid simulation inputs without panicking.
- Skeleton bindings store stable scene entity identity plus authored bone paths. They do not promise that a selected backend supports ragdolls or articulated constraints.
- The optional simulation contract may depend on this always-on schema. The scene schema must never depend on the optional simulation contract.
- The old owner files, imports, docs, and structure-test expectations must disappear in the same public cutover.

## Test Coverage

The mounted module tests cover material JSON round-trip, sparse joint-limit TOML, compact defaults, the map form, the three-slot sequence form, and rejection of duplicate/unknown/oversized axis input. The 2026-07-11 current Runtime test binary executed the complete `physics` filter as 35 passed / 0 failed; this includes three direct scene-schema tests plus scene asset, artifact, project IO, reflection, and framework-sync consumers.

The Frameworks 03 static suite passes 27/27, including the six Physics ownership/feature/adapter gates. Nightly `core-min + physics-contracts` passes independently in 12m39s with 52 existing warnings, and nightly `target-server` passes in 15m14s with 53 existing warnings. Full Frameworks M1 profile/App gates remain tracked by the parent plan and are not implied by this module-level acceptance.

## Plan Sources

This ownership cut is part of Frameworks 03 M1 and implements the plan rule that feature gates belong at module declaration and assembly boundaries. It also applies the code-structure convention by keeping the new root structural and separating declaration from serialization behavior.

## Open Work

- No scene-schema hard-cut work remains in this slice.
- Backend-specific validation, native constraint coverage, and later authored-shape expansion remain Physics plugin milestones; they must extend this owner rather than recreate a plugin-local persistence schema.
