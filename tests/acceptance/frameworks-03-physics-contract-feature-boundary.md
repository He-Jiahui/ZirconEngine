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
status: green-production-hard-cut-validated
---

# Frameworks 03 Physics Contract Feature Boundary Acceptance

## Scope

This record covers the test-first boundary and production hard cut for the Frameworks 03 M1 Physics slice. The complete twelve-domain matrix has subsequently passed; this record does not claim that the Runtime/App full suites or Frameworks M1 as a whole has passed.

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

## GREEN Evidence

The production hard cut is mounted atomically:

- Runtime/App expose `physics-contracts`; Client/Editor presets include it and Server does not.
- `core::framework::physics`, manager resolver/holder/service names, LevelSystem enabled adapter, and diagnostics enabled collector share the same declaration gate.
- persisted material/joint/skeleton schema has one always-on `core::framework::scene::physics` owner; old declaration files and old public imports are absent, with no re-export or alias.
- LevelSystem and Runtime diagnostics select enabled/disabled child owners at declarations; the common owners do not import optional Physics contracts.
- the Physics runtime plugin explicitly requests `physics-contracts` and remains the concrete simulation/backend owner.

Fresh 2026-07-11 evidence:

- Frameworks 03 contract/server/matrix static suites: 27 passed / 0 failed in 26.19s;
- current Runtime `physics` filter: 35 passed / 0 failed in 23.45s;
- nightly locked/offline `core-min + physics-contracts`: passed in 12m39s with 52 existing warnings;
- nightly locked/offline `target-server`: passed in 15m14s with 53 existing warnings after waiting for the shared target lock;
- Physics plugin owner evidence: feature-on 46/46 and feature-off 43/43 locked/offline suites, including builtin/Jolt queued-force and unchanged-body anchors;
- old persistent-schema owner and Activity-independent Physics path scans are enforced by the static gate rather than a compatibility allowlist.

The first nightly matrix attempt stopped before compilation because root `Cargo.lock` did not yet include the active Animation runtime manifest's `serde/toml` dependencies. Offline root metadata synchronization added only those declared dependencies; the identical locked command then passed. This is recorded as lock drift, not a Physics source failure.

The plan-output audit currently reports 23 violations outside Frameworks (priority overview, root index, Editor UI, and Render children) and zero Frameworks violations. Those external active-owner records were not modified by this slice.

## Current Decision

Status is `GREEN / production hard cut validated` for the Physics slice. The complete twelve-domain matrix is now green; Frameworks 03 M1 remains in progress until the declared Runtime/App full testing stage is green. No Physics-specific production, ownership, single-domain, or Server-exclusion blocker remains.
