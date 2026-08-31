---
related_code:
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/asset/assets/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/reflection/rigid_body.rs
  - zircon_runtime/src/scene/world/property_access/entries/physics.rs
  - zircon_runtime/src/scene/world/property_access/write/physics.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/BodyInstanceCore.h
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/Chaos/ChaosPhysicalMaterial.h
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Private/BodyInstanceCore.cpp
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-and-resource-lifecycle.md
status: static_complete_dynamic_pending
---

# Runtime scene physics schema current-source review (2026-08-30)

## Scope and status

`zircon_runtime/src/core/framework/scene/physics/**` was read file by file: 11 Rust files, 423 physical lines, 376 nonempty lines and 13,960 bytes, including 3 focused serde/validation tests. Sorted raw-content aggregate SHA256 is `ade5f4529215e41fc320dac951e746b0ccb625d1cd3fe6329452fb2032e3f955`. The production files are clean in the current worktree. Direct rustfmt was attempted for all 11 files; it reports only the existing import-order difference in `joint_constraint_metadata.rs`, with no logic or generated-code issue.

This directory is scene-facing authored schema, not a frame-loop owner. It contains no locks, threads, channels, filesystem/network I/O, WGPU calls or unbounded runtime queue. It remains covered by the broad `zircon_runtime/src/core/**` row in `pending.md`; static coverage does not qualify it for `review.md` because current-source Cargo, scale counters and F0/F2/F4 product evidence remain blocked.

## Findings

The enums and fixed-size records (`PhysicsCcdMode`, `PhysicsCombineRule`, `PhysicsJointDrive`, `PhysicsMassProperties` and `PhysicsSleepPolicy`) are Copy-sized policy values. `PhysicsJointConstraintMetadata` stores three-axis optional limits and fixed three-axis drives; its custom serde supports both sparse x/y/z maps and legacy arrays, rejects duplicate/unknown/overlong axes, and allocates only at explicit document serialization. `PhysicsMaterialMetadata` and `PhysicsSkeletonJointBinding` are authored records; their owned strings are appropriate at asset/scene boundaries.

`PhysicsMassProperties::is_valid` performs finite/positive-density and 3x3 positive-definite checks, but a current-source search found no production caller. Validation therefore is not currently a measured per-frame cost; the missing contract is that body/collider preparation should validate once and publish a typed ready/fault fact rather than repeatedly reconstructing or copying the schema.

The direct consumers create a second ownership layer in `core/framework/physics` sync states and scene components, while property access maps enum values to newly allocated strings for explicit reflection/export operations. Asset scene records and runtime scene components both retain the same metadata shape. Those copies are selected authoring/sync work, not a default-frame hotspot, but they leave schema validity and generation authority split between asset, scene, reflection and backend state.

## Reference-engine constraint

Unreal `FBodyInstanceCore` keeps authored simulation policy such as mass override, start-awake and mass-dirty flags together in the body contract (`dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/BodyInstanceCore.h`, around lines 33-66). Chaos physical material keeps friction, restitution and sleeping thresholds as one material state (`.../ChaosPhysicalMaterial.h`, around lines 33-60), and `BodyInstanceCore.cpp` initializes the mass-dirty policy once. This supports one typed body/material configuration authority with explicit dirty transitions; it does not prove Zircon should copy Unreal's bitfield ABI.

## Architecture handoff

- M0: add body/collider prepare counters for schema validation, property-string materialization and metadata copies; cover 0/1/1k bodies and invalid/nonfinite values.
- M1: compile one immutable `PhysicsSchemaGeneration` for material, body, joint and collider policies. Asset, ECS, reflection and backend sync borrow typed views and share one generation.
- M2: validate mass/inertia, axis limits and material ranges in a bounded prepare proposal before backend mutation. Invalid candidates return typed errors and do not publish partial sync state.
- M3: keep serde compatibility at explicit import/export boundaries; property access uses compact enum/field IDs in stable paths and materializes strings only for admitted exports.
- M4: qualify backend body/joint state by world/device/provider generation and use dirty transitions so unchanged physics metadata is not recopied into every sync pass.

## Acceptance gates

Dynamic acceptance requires current-source Cargo plus scale evidence for asset load, ECS sync, reflection access and backend preparation: validation visits, metadata/property-string allocations and bytes, schema copies, dirty versus unchanged bodies, and invalid-candidate rollback. Hard gates are one accepted schema authority, one validation per changed generation, zero unchanged-frame schema copies, typed invalid/nonfinite outcomes, preserved sparse/legacy serde behavior, and diagnostics that match actual materialization. No local leaf optimization is justified before those cross-owner gates.
