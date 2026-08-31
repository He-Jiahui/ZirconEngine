# Runtime 04 scene cache payload owner split

## Scope

- Target: `zircon_runtime/src/asset/artifact/cache_payload/scene.rs`.
- Baseline: clean 779-line current-source production owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 04, and the existing artifact scene round-trip suite.
- This slice changes source ownership only. It does not change the artifact manifest version, bincode field/variant order, scene semantics, or claim a cache-time or power improvement.

## Architecture review

The previous file placed the scene cache envelope, entity aggregation, mesh/camera projection, rigid-body/collider/joint projection, and script JSON conversion in one owner. These domains already change independently in the public scene asset model, so retaining a second monolithic cache representation made every component evolution review the entire wire owner.

The primary local Unreal references were `CoreUObject/Public/UObject/Object.h`, `Engine/Classes/GameFramework/Actor.h`, and `Core/Public/Serialization/StructuredArchive.h`. Unreal keeps serialization at the object boundary while actor component domains retain their own data and lifecycle responsibilities. Zircon keeps one scene artifact envelope and one bincode stream, but gives each component family a typed conversion owner.

The repository's canonical `asset/assets/scene/` family already separates `entity`, `camera`, `mesh`, `physics`, and `extensions`. The cache payload split follows the same domain model without exposing cache-only DTOs, changing the public scene API, or adding a compatibility representation.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `scene.rs` | Serialized scene envelope and top-level scene conversion | 38 |
| `scene/entity.rs` | Serialized entity aggregate and component-family composition | 136 |
| `scene/rendering.rs` | Mesh primitive/LOD/instance and camera/target conversion | 231 |
| `scene/physics.rs` | Rigid body/mass, collider/shape, and joint/constraint conversion | 358 |
| `scene/script.rs` | Script binding and JSON table conversion | 44 |

The parent `cache_payload.rs`, artifact manifest/version owner, and the currently modified artifact test root were not touched. `ArtifactCacheSceneAsset` retains the same crate-scoped visibility and parent import path.

## Wire and behavior invariants

- All 14 serialized struct/enum names, field order, field types, serde attributes, and enum variant order are unchanged.
- The scene envelope still serializes one ordered entity vector and reconstructs entities in the same order.
- Camera, mesh, rigid body, collider, joint, and script conversions retain the same clone/move/error behavior.
- Recursive compound collider shape conversion and script JSON failure propagation remain unchanged.
- Artifact manifest schema/version, compression, chunking, and cache publication code are outside this slice and unchanged.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the root and four child owners.
- Scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static type-block comparison retained all 14 serialized definitions byte-for-structure after removing only visibility and whitespace differences; mismatches were zero.
- Static function comparison retained all 28 conversion function definitions and name multiplicities.
- Production owners contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Root size changed from 779 to 38 lines; all production owners are at or below 358 lines.
- Managed Cargo and artifact round-trip execution were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because serialization work, allocations, and wire bytes were intentionally unchanged. Any later cache optimization requires an exact scene-size/encode/decode/allocation baseline and artifact byte-compatibility proof before implementation.
