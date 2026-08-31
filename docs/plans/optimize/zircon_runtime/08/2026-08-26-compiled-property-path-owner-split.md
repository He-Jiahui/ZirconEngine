# Runtime 08 compiled property-path owner split

## Scope

- Target: `zircon_runtime/src/scene/world/compiled_binding/property_path.rs`.
- Baseline: clean 700-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 08, and `failure-2026-07-22-scene-property-path-compiled-dispatch.md`.
- This slice changes source ownership only. It does not replace the compiled-binding algorithm, claim a frame-time or power improvement, or close Runtime 08 acceptance.

## Architecture review

The previous file mixed four independently changing responsibilities: compiled identity and typed writer models, import/edit-boundary resolution, stable-frame reads, and stable-frame writes with dirty-generation publication. That made the correct compile-once/apply-many design harder to audit and forced unrelated component read/write changes through one 700-line owner.

The primary local Unreal reference was `dev/UnrealEngine/Engine/Source/Runtime/PropertyPath/Public/PropertyPathHelpers.h`. Unreal separates serialized path segments, resolution, and cached fast access through `FPropertyPathSegment` and `FCachedPropertyPath`. The Rust reference was Bevy's `AnimationTargetId`, `AnimatableProperty`, `AnimatedField`, and component-field evaluator identity under `dev/bevy/crates/bevy_animation/src`. Both support retaining stable identities and typed accessors after the compile boundary.

Zircon already follows that algorithmic direction: `PathId`/`ComponentFieldId` and generation validation own stable identity, while a typed enum performs steady-state dispatch. The reviewed bottleneck in this slice was ownership coupling, not lookup complexity. The implementation therefore preserves the algorithm and separates its lifecycle owners.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `property_path.rs` | Private child mounts and compatible public re-exports | 8 |
| `property_path/model.rs` | Stable IDs, compiled targets, typed writer variants, and generation-currentness checks | 218 |
| `property_path/compile.rs` | Entity/path resolution, canonical identity interning, typed writer compilation, and root discovery | 135 |
| `property_path/read.rs` | Stable-frame typed property read dispatch | 162 |
| `property_path/write.rs` | Stable-frame typed writes, component mutation, cache dirtying, and generation publication | 219 |

External users continue importing `CompiledScenePropertyTarget`, `CompiledScenePropertyWriter`, `ComponentFieldId`, and `PathId` through the existing `compiled_binding` and `scene::world` routes. Internal tuple construction and `property_path()` access retain their original `compiled_binding` visibility after the nested-module move.

## Behavior and complexity invariants

- Compile boundaries still canonicalize and intern the entity/property identity once.
- Stable reads and writes still dispatch through the same typed enum variants and do not add path parsing, path lookup, reflection enumeration, heap allocation, or string fallback.
- Scene/schema currentness rejection, component errors, transform/camera/mesh/light/dynamic dispatch, and world-generation publication are unchanged.
- `scene_binding_root` retains its bounded parent walk and cycle guard; no topology algorithm was altered.
- The existing source-contract test now inspects the facade, model, compile, and write owners separately while preserving unrelated dirty test edits.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the facade, four child owners, and the adjusted source-contract test.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notices.
- Static migration comparison retained all 23 function definitions and all 7 type definitions from the original owner.
- Static typed-variant comparison found identical occurrence counts for all transform, mesh, animation, camera, light, and dynamic variants.
- Production files contain no `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Root size changed from 700 to 8 lines; all production child owners are at or below 219 lines.
- Managed Cargo and runtime performance validation were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because the compiled-dispatch algorithm and operation count were intentionally unchanged. Any later property-binding optimization must first capture compile-boundary cost, stable read/write throughput, allocation counters, and representative frame/power evidence, then demonstrate that the measured bottleneck is removed.
