# Runtime61 M0 Play snapshot exact-conservation characterization

- Owner: `runtime61-play-snapshot-characterization-r1-f0f9ae6f-20260824`
- Source plan: `61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md`
- Findings: `RWL-P0-003`, `RWL-G03`, `RWL-P1-060`
- Status: source characterization implemented; managed RED execution compile-blocked before the
  target test; M0 remains partial

## Completed in this slice

- Added a product-path characterization that renders a real project fixture, writes an empty
  versioned `.zrscene.json`, starts a headless dynamic runtime session with that Play override, and
  requires the loaded runtime World to contain exactly zero entities.
- The fixture root is derived from the test executable directory, so generated data remains under
  the managed Cargo target rather than the system drive.
- Kept the test separate from the already busy shared dynamic-session test tree and left production
  restore behavior unchanged. The expected current RED is the three entities inserted by
  `World::new()`: Camera, DirectionalLight, and Cube.
- Marked the characterization `#[ignore]` with an explicit M0 RED reason. It remains available for
  direct opt-in execution without making an unfinished architecture contract part of the routine
  library-test gate.

This is one of the five required M0 characterization groups. It does not close `RWL-P0-003` or
authorize the production hard cut.

## Architecture decision

Current `DynamicScene::spawn_into` already compiles an `EntityRemap`, validates the target World
generation/schema/component-registry/change-tick, stages the mutation, and commits it atomically.
The structural defect is above that transaction: `load_play_scene_level` creates a template
`World::new()` and then appends the snapshot. The future implementation must introduce a dedicated
runtime restore/fork artifact and an exact-conservation receipt. It must not change the general new
scene template semantics or silently discard the remap.

The Unreal reference keeps the same boundary explicit: `UWorld::CreateWorld` initializes a new
World, while PIE uses `DuplicateWorldForPIE` or package loading and then publishes the loaded World.
This supports separate new-scene and restore constructors rather than one constructor with implicit
template entities.

## Play snapshot writer/loader inventory

This inventory is intentionally limited to the versioned DynamicScene Play path. The canonical
authoring `.scene.toml` inventory remains owned by the other M0 groups.

| Disposition | Current owner/data | Evidence and consequence |
| --- | --- | --- |
| Included | Entity source IDs and complete current `NodeRecord` projection | Capture walks stable `node_records`; the record includes hierarchy, transform, 3D/2D render, light, physics, and animation fields. |
| Included | Serializable reflected component fields with component adapters | Capture visits registered component runtimes, filters to serializable fields, and restore resolves writable field slots before commit. |
| Included | Plugin-owned dynamic components | Capture emits the matching component descriptor; restore resolves the type path and reconstructs the JSON field object. |
| Included | Serializable reflected resources with resource adapters | Capture records sorted resources; restore requires an existing resource or an `ensure` callback and applies staged fields. |
| Skipped | Registrations not marked serializable and fields not marked serializable | Capture deliberately omits them, but the artifact currently has no participant receipt that reports the omission. |
| Skipped | Metadata-only component/resource registrations without an adapter | Capture continues past them, again without an Included/Skipped/Unsupported receipt. |
| Skipped | Reflected fields that are serializable but not editable at the target | Restore resolves them but does not write them; no receipt exposes that disposition. |
| Unsupported | Unknown type/field, address-kind mismatch, missing target adapter/resource, incompatible descriptor, invalid source graph, exhausted entity ID | Compile/preflight returns typed errors and does not publish the staged mutation. |
| Unsupported | Exact persistent identity and general entity-reference mapping | The artifact persists live `EntityId`; remapping covers the current hand-maintained fields rather than a schema-owned mapper. |
| Missing contract | Conservation receipt | `EntityRemap` is returned internally, but project startup discards it and publishes no source/restored entity/resource counts or participant dispositions. |

## M0 status

| Required characterization group | Status | Owner boundary |
| --- | --- | --- |
| Arbitrary typed/dynamic component Save/reopen | Pending | Editor Save plus Runtime61 schema/snapshot owner |
| Play enter/exit exact authoring conservation | Pending | Editor07 Play lifecycle plus Runtime61 runtime-fork owner |
| Empty Play snapshot has no default entities | Source complete; managed execution compile-blocked before test | Runtime61 dynamic project startup |
| Terrain/tilemap/prefab canonical roundtrip | Pending | Runtime61 scene schema/provider owner |
| Sprite2D/Mesh2D canonical roundtrip | Pending | Runtime61 scene schema/provider owner |

Writer freeze remains active: no fields were added to `World::clone`, `SceneAsset`, or the legacy JSON
writer. Production implementation remains blocked by the four missing M0 product characterizations
and by the unfrozen identity/schema/participant contracts.

## Validation evidence

The UI12 Cargo job was allowed to finish and its requested quiet window elapsed before this session
started a Windows-native managed test job. The target and command were:

- Job: `73f34c657b224332ac8de4cc3b33ae7c`
- Run: `477685b38f9546ebb9dfaa9ab290a88b`
- Target: `F:\cargo-targets\zircon-engine\ephemeral\test\73f34c657b224332ac8de4cc3b33ae7c`
- Command: `cargo +1.94.1 test -p zircon_runtime --lib
  empty_versioned_play_snapshot_restores_an_exact_empty_runtime_world -- --nocapture`
- Result: exit `101`; compilation failed with 92 errors and 1463 warnings before the target test
  executable was produced. The job finished naturally and was released with coordinator cleanup.

The 92 errors cluster in nine foreign current-source areas, so they are not evidence that the
Runtime61 assertion reached RED:

| Error group | Count | Foreign source area |
| --- | ---: | --- |
| Native plugin loader import/type inference | 55 | `tests/plugin_extensions/native_plugin_loader.rs` and `real_fixture.rs` |
| Clock-domain test imports | 10 | `core/framework/tests/framework_surfaces.rs` |
| Cubemap direction/solid-angle helpers | 6 | `core/framework/render/environment/source_cubemap/tests.rs` |
| Schedule-runner test support/API drift | 10 | `scene/ecs/schedule_runner/tests/` |
| Realtime IBL test module path | 4 | `graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs` |
| Offline font-bake result API drift | 3 | `text/sdf/font_bake/tests/offline.rs` |
| Core runtime plugin-resolution API drift | 2 | `core/runtime/tests/resolution/behavior/factory_panics.rs` |
| UI hotspot counter visibility | 1 | `core/runtime/diagnostics/profiling/ui_hotspot/tests/mod.rs` |
| UI action invocation route field drift | 1 | `ui/tests/event_routing/component_events/missing_policy.rs` |

The compiler summary is therefore recorded as `compile_blocked_before_test`, not as an observed
Runtime61 RED. A future opt-in rerun must first use a current-source snapshot in which those foreign
test surfaces compile.
