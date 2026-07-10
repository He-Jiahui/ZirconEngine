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

The target files are staged but not yet mounted from `core::framework::scene::mod.rs`. The active hard-cut slice intentionally keeps them non-public while other sessions compile the current Runtime source graph. The production cutover is incomplete until every consumer imports this owner directly and the superseded declarations under `core/framework/physics` are deleted.

No compatibility re-export from the old Physics root is permitted. A state in which both paths are public is invalid.

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

The staged module tests cover material JSON round-trip, sparse joint-limit TOML, compact defaults, the map form, and the three-slot sequence form. They are not yet compiled because the module is intentionally unmounted during the active implementation window.

The Frameworks 03 static boundary test was observed RED before implementation and remains RED at the expected missing `pub mod physics;` mount. The milestone testing stage must later prove:

- the static boundary is fully green;
- `core-min` can use scene schema without Physics simulation contracts;
- `physics-contracts` compiles alone with `core-min`;
- `target-server` excludes the optional Physics contract;
- the Physics plugin compiles with an explicit feature request;
- scene, asset, editor, Runtime, App, and full domain-matrix gates pass.

## Plan Sources

This ownership cut is part of Frameworks 03 M1 and implements the plan rule that feature gates belong at module declaration and assembly boundaries. It also applies the code-structure convention by keeping the new root structural and separating declaration from serialization behavior.

## Open Work

- Mount the scene schema and update every caller directly.
- Gate the optional Physics framework and manager surfaces.
- Install enabled/disabled declaration adapters for LevelSystem state and Runtime diagnostics collection.
- Replace the editor diagnostic fixture with the neutral Runtime projection.
- Delete the superseded Physics schema files and remove old-path references.
- Run the declared WSL-first testing stage and promote the acceptance record only after all gates pass.
