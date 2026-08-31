---
related_code:
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/capture.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/Class.h
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
status: static_complete_dynamic_pending
---

# Runtime scene component type descriptors current-source review (2026-08-30)

## Scope and status

`zircon_runtime/src/core/framework/scene/component_type_descriptor/**` was read file by file: 4 Rust files, 67 physical lines, 56 nonempty lines, 1,753 bytes, no inline tests, no I/O, locks, threads, channels, WGPU calls or frame-loop work. Per-file SHA256 values are `component_property_descriptor.rs` `2b224c51901402dd26e39ac2a784736354667089bfa6b6d7f6afe2ee589ed3cc`, `component_type_descriptor.rs` `2a5966ac45195155c64f9fa3354f5ba5c1595b9f44e0d976bf75072d81fbedef`, `constructors.rs` `84ddc59b14d92de881c79b3f71263c645097c5bc17b53ac1072fdcf950451c3d`, and `mod.rs` `954aeb622ab86ea101c05b53e8c41ecda2950c5a981404bc714293e2ec359c2e`; sorted raw-content aggregate SHA256 is `c94447c5ababb04043738b8d8f6c2a43824daa6e06be3fc969d7c2cce8b1b189`. Direct `rustfmt --check --edition 2024 --config skip_children=true` passes 4/4.

This is static coverage only. The module remains under the existing `zircon_runtime/src/core/**` pending entry and does not enter `review.md`; current-source Cargo, scale counters and F0/F2/F4 product traces remain unavailable behind the known workspace blockers.

## Findings

The leaf descriptors are intentionally owned serde DTOs: a type owns `type_id`, `plugin_id`, `display_name` and a `Vec<ComponentPropertyDescriptor>`, while each property owns its name, value type and editability. Constructors allocate this schema once on registration/configuration and contain no repeated runtime work. No safe local micro-optimization is justified.

The cost and authority split occurs at consumers. `World::register_component_type` first derives a reflection registration from the descriptor, then publishes the descriptor into `ComponentTypeRegistry` and a second metadata graph into `TypeRegistry`. The same component schema is therefore retained in two registries, with registration-time string/property copies. Dynamic-scene capture builds a temporary `Vec<&ComponentTypeDescriptor>` and then deep-clones descriptors selected by required type IDs for its owned document payload; this is explicit export work, not a default-frame hotspot. Public dynamic-component queries similarly clone a descriptor into each owned DTO.

`ComponentTypeRegistry` also maintains a separate per-type schema map and a catalog counter. `advance_schema_generation` uses saturating arithmetic, so exhaustion can alias later generations. This is an adjacent runtime/world ownership issue, not a leaf-constructor fix: the descriptor, reflection adapter and schema generation need one immutable accepted generation before a direct change is safe.

## Reference-engine constraint

Unreal `UStruct` retains one linked `ChildProperties` chain and exposes `FindPropertyByName`/visitor access in `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/Class.h` around lines 522, 663 and 960-980. That is evidence for one reflected type metadata authority with borrowed traversal. It does not prescribe Zircon's serde ABI, but it supports replacing parallel owned schema graphs with an Arc-backed descriptor generation and dense property handles.

## Architecture handoff

- M0: add registration/export counters for descriptor clones, property bytes, temporary reference vectors and schema-generation exhaustion; add 1/71/256/1k descriptor scale cases.
- M1: compile one immutable `ComponentSchemaGeneration` containing provider/plugin, schema/toolchain, descriptor and property handles. Reflection and scene serialization borrow it.
- M2: validate a candidate registry generation before publication; duplicate/conflicting descriptors and generation exhaustion return typed errors without changing the accepted generation.
- M3: make dynamic-scene capture traverse the accepted generation by required IDs and materialize owned descriptors only in an explicitly byte-admitted export operation; stable consumers clone zero.
- M4: replace saturating counters with checked device/world-qualified generations and qualify compiled writers, dynamic queries and scene payloads by that generation.

## Acceptance gates

Dynamic acceptance requires current-source Cargo plus scale evidence for registration, export and query paths: descriptor/property clone count and bytes, temporary collection allocations, schema lookup probes, 1/4/16 viewport or scene consumers, and generation exhaustion. The hard gates are one accepted schema authority, zero stable-generation deep clones, typed conflict/exhaustion outcomes, unchanged serialized descriptor semantics, and diagnostics that match actual materialization. Until those gates run, this module stays pending.
