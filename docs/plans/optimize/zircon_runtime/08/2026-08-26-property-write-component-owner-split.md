# Runtime 08 property-write component owner split

## Scope

- Target: `zircon_runtime/src/scene/world/property_access/write.rs`.
- Baseline: clean 610-line current-source production owner before this slice, with the existing 442-line `write/physics.rs` owner left unchanged.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 08, Runtime 15, and `failure-2026-07-22-scene-property-path-compiled-dispatch.md`.
- This slice changes source ownership only. It does not replace property normalization or dispatch, claim a frame-time or power improvement, or close scene-property acceptance.

## Architecture review

The previous `set_property_impl` was a 530-line route that owned node metadata, hierarchy, Transform, camera, mesh, five light types, four animation player families, physics delegation, and dynamic fallback. The slow-path facade was therefore also the concrete mutation owner for unrelated component domains.

The primary local Unreal references were `Runtime/PropertyPath/Public/PropertyPathHelpers.h` and the component owners `Camera/CameraComponent.h`, `Components/MeshComponent.h`, and `Components/LightComponent.h`. Unreal keeps property-path resolution/cached access separate from camera, mesh, and light mutation semantics. Zircon now follows that owner direction while retaining `World::set_property` as the public path facade and preserving the current ECS storage API.

The Runtime08 compiled writer remains the stable-frame route. This generic writer is an inspection/edit slow path and still normalizes the component and property segments exactly once before dispatch. No attempt was made to optimize that path without profiling evidence.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `write.rs` | Public entry, generation publication, one-time normalization, core node/Transform route, and dynamic fallback | 170 |
| `write/animation.rs` | Skeleton, clip/sequence, graph, and state-machine player writes | 194 |
| `write/camera.rs` | Camera field writes | 48 |
| `write/lighting.rs` | Ambient, directional, point, rect, and spot light writes | 229 |
| `write/mesh.rs` | Mesh resources, ordering, morph weights, tint, and typed read-only/index errors | 127 |
| `write/physics.rs` | Existing rigid-body, collider, and joint writes | 442 |

All child entry points are restricted to the parent `write` module. Public callers continue to use `World::set_property`; there is no compatibility facade or parallel implementation.

## Behavior and complexity invariants

- Missing-entity rejection still occurs before normalization and domain dispatch.
- Component and property segments are still normalized once with a pre-sized `Vec`.
- Root dispatch retains all original component aliases and the dynamic-component fallback remains last.
- All no-change exits, typed errors, resource writes, morph-weight resizing, cache dirtying, and final inspection/world generation publication are unchanged.
- Transform stays with the core route because it uses the World mutation API that owns hierarchy/cache publication; camera, mesh, light, animation, and physics remain component-domain leaves.
- Stable-frame animation application continues to use compiled writers and is not redirected through this slow path.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for five production files and four adjusted contract-test files.
- Static literal comparison retained all 91 old string literals with no missing or additional literal.
- Static branch comparison retained identical counts for `ScenePropertyValue` (4), `SceneError` (7), cache-dirty calls (12), no-change returns (31), resource-handle construction (5), and dynamic fallback (1).
- Root and child source-contract anchors are complete; the existing pre-sized normalization and typed-error guards now inspect the owning files.
- Production owner files contain no `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Root size changed from 610 to 170 lines; all component-domain owners are at or below 442 lines.
- Managed Cargo and runtime profiling were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No CPU, allocation, memory, energy, or power improvement is claimed. A later slow-path optimization requires a representative Inspector/edit workload and allocation/profile baseline; stable-frame work must be measured separately against the compiled writer path.
