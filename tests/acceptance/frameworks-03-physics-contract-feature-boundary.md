---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/scene/level_system.rs
  - zircon_plugins/physics/runtime/Cargo.toml
implementation_files:
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_disabled.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_enabled.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_disabled.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_physics_contract_feature_is_forwarded_by_client_presets tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_physics_contract_declarations_are_feature_gated tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_physics_plugin_explicitly_requests_the_contract tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_persistent_physics_schema_has_one_always_on_scene_owner tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_optional_physics_runtime_state_uses_declaration_adapters tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_runtime_physics_diagnostics_use_a_neutral_projection_adapter
  - zircon_runtime/src/core/framework/scene/physics/tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_collection_enabled.rs
doc_type: acceptance-evidence
status: red-production-pending-staged-unmounted
---

# Frameworks 03 Physics Contract Feature Boundary Acceptance

## Scope

This record covers the test-first boundary for the pending Frameworks 03 M1 Physics slice. It does not claim that the production hard cut, Cargo matrix, or complete M1 testing stage has passed.

The accepted target architecture is:

- Runtime/App expose an independent `physics-contracts` feature; Client and Editor include it, while Server does not.
- Optional simulation, query, backend-status, world-sync, and manager contracts remain under `core::framework::physics` and compile only with `physics-contracts`.
- Persisted authoring DTOs move to the always-on `core::framework::scene::physics` owner: material metadata, combine rule, joint constraint metadata, joint drives, and skeleton joint bindings.
- The old declaration files and old public paths are deleted in the same cutover. No re-export, alias, facade, shim, or runtime fallback is allowed.
- `LevelSystem` and runtime diagnostics select enabled/disabled child modules at declaration boundaries. Always-on diagnostics expose a neutral projection rather than `PhysicsBackendStatus`.
- `PhysicsJointConstraintMetadata` keeps its declaration separate from sparse-axis serialization behavior so the new owner does not reproduce the old mixed-responsibility file.

## Reference Evidence

- Godot primary: `dev/godot/scene/resources/physics_material.{h,cpp}` owns persisted material data separately from `dev/godot/servers/physics_3d`; `dev/godot/tests/scene/test_physics_material.cpp` verifies resource defaults and authored values.
- Fyrox secondary: `dev/Fyrox/fyrox-impl/src/scene/{collider,joint}.rs` owns scene-facing material/joint data, while `scene/graph/physics.rs` owns the simulation world and synchronization path.
- Unreal scale check: `dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/PhysicalMaterials/PhysicalMaterial.h` keeps friction, restitution, and combine policy in a dedicated physical-material owner rather than a backend manager facade.

Zircon deliberately places the persisted schema under framework scene contracts instead of a separate root package, preserving the fixed `zircon_app` / `zircon_runtime` / `zircon_editor` architecture.

## RED Evidence

The initial exact six-test run completed as six assertion failures and zero read/setup errors:

- Runtime has no `physics-contracts` feature.
- Physics framework and manager declarations are unconditional.
- Physics runtime plugin does not explicitly request the contract.
- `core::framework::scene::physics` does not exist and persisted DTOs remain in the optional owner.
- `LevelSystem` directly stores optional Physics contract types.
- Runtime diagnostics directly exposes `PhysicsBackendStatus` and resolves the Physics manager in the shared collector.

The target scene schema, separated joint serde behavior, LevelSystem feature-on/off runtime-state adapters, neutral diagnostics DTO, and feature-on/off diagnostic collectors are now staged as unmounted files. Their scoped rustfmt and whitespace checks pass. Focused re-execution has advanced to the intended integration failures: the scene owner is not mounted from `scene/mod.rs`, `level_system.rs` still imports/stores the optional Physics contract directly, and `physics.rs` still embeds `PhysicsBackendStatus`. No staged file participates in the active Cargo source graph and no new public owner exists yet.

Staged unit coverage includes scene material JSON roundtrip, default and sparse joint TOML roundtrip, three-slot JSON axis input, rejection of duplicate/unknown axes and arrays longer than three slots, plus complete neutral backend-state and simulation-mode projection. These tests are parsed by rustfmt but cannot execute until the modules are mounted; no pass is claimed yet.

Production implementation remains pending while `20260710-1920-plugin-architecture-continuation` owns Physics manager imports and active Runtime/Editor/PBR Cargo lanes compile shared source. The hard cut must mount the new files, delete the old declarations, migrate every consumer, and add the feature gates atomically.

The plan-output audit currently reports 23 violations outside Frameworks (priority overview, root index, Editor UI, and Render children) and zero Frameworks violations. Those external active-owner records were not modified by this slice.

## Current Decision

Status is `RED / production pending / target files staged but unmounted`. This is a completed test-contract slice only. The plan must remain M1 in progress until the hard cut, old-path scan, standalone Physics check, Server exclusion, Physics plugin check, App checks, and full domain matrix are green.
