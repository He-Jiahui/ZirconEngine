---
related_code:
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/core/framework/scene/entity_path
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/core/framework/scene/mobility
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/animation/target_id.rs
  - zircon_runtime/src/scene/world/compiled_binding
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SceneComponent.h
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
status: static_complete_dynamic_pending
---

# Runtime scene path and mobility current-source review (2026-08-30)

## Scope and status

The scene path and mobility scope contains 4 Rust files: `entity_path.rs`, its 96-line focused test module, `mobility.rs`, and its 126-line focused test module. The current aggregate is 513 physical lines, 431 nonempty lines and 15,041 UTF-8 bytes; sorted raw-content SHA256 is `2c612c6840e17273ca2a4e91c693e352c8cbe75baed6f7c4aa59c5d9e455d54b`. The two production files and both split test files are currently foreign modified/untracked work and were not edited. Direct rustfmt is 2/4: the remaining failures are existing benchmark-print wrapping and import ordering.

This is a scene/animation schema boundary, not a default frame-loop owner. `EntityPath::parse` performs one split/filter/map pass with a bounded capacity estimate, `ComponentPropertyPath` constructs its canonical raw form once, and `Mobility` is a two-value `Copy` enum whose reflection parser uses trim plus `eq_ignore_ascii_case` without allocating a lowercase string. Focused tests preserve legacy parse results and fence the allocation-free parser shape.

## Findings

The local leaf implementation has no locks, threads, I/O, WGPU calls or unbounded queue. Existing direct consumers show where its cost can become selected work: `AnimationTrackPath::split`, `entity_path` and `property_path` reparse the owned raw track string into new path objects on each accessor; animation target compilation hashes path segments; scene/property and asset authoring boundaries own path strings for serialization and reflection. These are explicit import/compile/query operations, not proof of a per-frame hotspot, but repeated access can rebuild equivalent path trees when a compiled binding generation is available.

The render relevance path consumes `Mobility` as a compact value and does not stringify it. Authoring assets and reflection/property export intentionally convert enum/path values to owned text. The safe optimization boundary is therefore generation reuse at animation/scene binding consumers, not a local change to the schema structs. A stable `AnimationTrackPath` or compiled binding should expose borrowed component/entity segments and cache the parsed target identity; owned strings remain only at import/export or an admitted diagnostic/export boundary.

## Reference-engine constraint

Unreal's `FSoftObjectPath` stores a structured top-level asset path plus an optional subobject path and explicitly avoids string conversion when a resolved object already exists (`dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h`). `USceneComponent` stores attachment parent/socket as typed `FName` values rather than rebuilding path strings for every scene operation (`.../Engine/Classes/Components/SceneComponent.h`). This supports typed/stable path identity with explicit string conversion at boundaries; it does not imply replacing Zircon's serde representation or animation path syntax.

## Architecture handoff

- M0: add path parse/access counters and scale cases for 0/1/1k segments, repeated track access, reflection export and mobility conversion; keep focused legacy-equivalence tests.
- M1: compile one immutable `ScenePathGeneration`/`AnimationBindingGeneration` with stable segment IDs, target IDs and borrowed views. Repeated animation evaluation and scene queries use the generation rather than reparsing owned text.
- M2: validate segment count, UTF-8 byte length and nesting/depth before authoring/import allocation. Return typed overflow/invalid-path outcomes without partial binding publication.
- M3: retain canonical serde/display strings only at explicit import/export and diagnostics boundaries; stable render relevance and reflection reads use compact enum/field IDs and do zero temporary string work.

## Acceptance gates

Dynamic acceptance requires current-source Cargo plus scale evidence for animation binding compilation/evaluation, scene property access and authoring import/export. Hard gates are legacy parse compatibility, bounded path bytes/segments, one generation-qualified binding identity, zero stable-frame reparsing/allocation, and diagnostics that match actual string materialization. No local production micro-fix is justified while the active path/mobility sources remain foreign-owned and the consumers are cross-owner generation work.
