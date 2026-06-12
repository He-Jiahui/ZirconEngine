# Vampire Dark Content Upgrade Design

## Summary

`examples/vampire` will move from a primitive arena demo to a darker, game-like 3D roguelite slice built from license-clear free assets. The playable contract remains intentionally small: WASD movement, third-person follow camera, automatic attacks, enemies chasing over the authored navigation mesh, and a survival balance loop. The visual contract becomes stricter: the project must contain real CC0 model/environment assets, a dressed dungeon/graveyard scene, authored dark materials, and actual runtime post-process settings rather than only shader tinting.

## Source And License Direction

The default art direction is a low-poly dark fantasy style because it is compatible with the current runtime asset importer and keeps the sample practical to ship in `examples/vampire`.

- KayKit Dungeon Remastered provides the primary modular dungeon set and is distributed as CC0 with GLTF/OBJ downloads.
- Kenney Graveyard Kit provides graveyard props and is distributed as CC0.
- KayKit Skeletons and KayKit Adventurers provide humanoid character/enemy meshes and are distributed as CC0.
- Quaternius Ultimate Monsters provides monster variants and is distributed as CC0 with GLTF/FBX/OBJ formats.
- Poly Haven remains an optional CC0 environment/HDRI/texture reference source; it is not required for the first playable pass if runtime skybox/HDRI binding would add scope.

Downloaded source archives and extracted files will live under `examples/vampire/assets/external/<source>/`. Converted/imported project-facing resources will live under the normal `assets/` and `library/` trees. `examples/vampire/LICENSES.md` must list every external pack name, source URL, author, license, and which local files came from it.

## Runtime Capability Boundaries

The current runtime can support a simple independent 3D game slice, but the design must stay inside the proven capability envelope.

- Supported now: directory projects, scene TOML import, material/model assets, GLTF/GLB model import, texture/material subassets, third-person camera transforms, keyboard input, scripted gameplay host calls, baked navmesh fallback movement, render extraction, and WGPU rendering.
- Supported but needs authoring glue: post-process effect stack exists in render extract/render framework, but scene TOML does not currently serialize `PostProcessSettingsComponent` or `PostProcessVolumeComponent`. This upgrade will add focused scene asset fields and world conversion before using them in the vampire scene.
- Not claimed as complete: GLTF animation channel playback. Current importer records animation/skin data as placeholders. The upgraded game may include animated source files in `assets/external`, but runtime-visible character movement will be script-driven transform motion until the animation system grows real GLTF clip playback.

## Game Content

The upgraded scene is a compact crypt arena, not an open-world game. It should feel like a real authored level: stone floors, walls/pillars, broken gates, coffins, tombstones, candles or braziers, and spawn lanes that make enemy approach readable from the third-person camera.

Core loop:

- Player moves with WASD.
- Camera follows from behind and above with stable framing.
- Blood Bolt auto-targets the nearest enemy in range.
- Enemies use `nav_move_towards_entity` toward the player over `assets/navigation/main.navmesh.toml`, falling back to direct movement when path data is unavailable.
- Enemy waves escalate with elapsed time through several authored enemy archetypes.
- Killed enemies can award script-tracked experience; level thresholds unlock deterministic stat upgrades in this first pass rather than a full UI picker.

Initial balance:

- Player: HP 120, move speed 5.2.
- Blood Bolt: 14 damage, 0.65 second cooldown, 9.0 range, one extra chained hit after level 3.
- Skeleton Grunt: HP 28, speed 2.8, contact damage 8.
- Bat/fast monster: HP 18, speed 4.4, contact damage 6.
- Caster/elite placeholder: HP 70, speed 1.7, contact damage 12, starts appearing after 90 seconds.
- Bruiser elite: HP 220, speed 1.2, contact damage 18, starts appearing after 180 seconds.
- XP thresholds: 5, 12, 22, 35, 55, then +25 each level.
- Upgrade order for the non-UI first pass: damage, cooldown, move speed, max HP, range, chain count.

## Rendering And Mood

The scene should read as dark fantasy from the first frame:

- Camera exposure and clear color tuned for a dark crypt.
- Global post-process settings with ACES/filmic tonemap, mild bloom, vignette, film grain, chromatic aberration, fog, and cool color grading.
- Warm emissive materials or point lights for candles/braziers, balanced against blue-gray ambient light.
- Materials remain simple PBR/runtime-compatible; avoid relying on advanced texture features that the example cannot validate.

If a specific post-process family cannot bind an authored resource yet, the settings should use renderer-supported fallback behavior and record that limitation in docs/tests rather than silently pretending the resource exists.

## Architecture And Files

Runtime scene post-process support:

- `zircon_runtime/src/asset/assets/scene.rs` gains serializable scene asset DTOs for camera post-process settings and post-process volumes.
- `zircon_runtime/src/scene/world/project_io.rs` maps those DTOs to and from `PostProcessSettingsComponent` and `PostProcessVolumeComponent`.
- `zircon_runtime/src/scene/tests/render_post_process_extract.rs` and/or asset scene roundtrip tests prove TOML load, world conversion, and render extract behavior.
- `docs/zircon_runtime/asset/assets/scene.md` is updated with the new fields.

Vampire project content:

- `examples/vampire/assets/external/` stores CC0 source packs and license notes.
- `examples/vampire/assets/models/`, `assets/materials/`, `assets/scenes/`, and `library/` are regenerated or extended so the scene references real imported meshes instead of only hand-authored capsules.
- `examples/vampire/scripts/vampire_game/main.zr` grows the wave/timer/XP/stat logic while staying small enough to understand.
- `examples/vampire/assets/data/balance.toml` stores the main balance values so tests and docs can inspect them.
- `examples/vampire/README.md` explains the upgraded content, capability boundaries, run command, and validation screenshot.
- `examples/vampire/LICENSES.md` records all external asset sources and local paths.

## Validation

Completion requires evidence stronger than “the files exist.”

- Project manifest, scene, scripts, materials, model references, navmesh, and license docs parse/import.
- Scene tests prove there are real environment mesh entities and multiple enemy archetypes.
- Post-process tests prove scene-authored settings enter render extract.
- Render project test proves visible mesh pixels are produced by the upgraded scene.
- Runtime launch with `--project E:\Git\ZirconEngine\examples\vampire` succeeds and a fresh screenshot shows the dressed dark scene.
- The existing WASD movement/session test remains valid or is updated to the new player/camera setup.

## Risks

External asset archive structure can change. The implementation should keep source URLs and license evidence in `LICENSES.md`, and if a pack download layout differs, choose a small subset of compatible GLTF/OBJ files rather than expanding scope.

GLTF animation playback is not part of this upgrade. Character movement can face/step through script transforms and visual scale/tint changes, but real skeletal animation should remain a documented future enhancement.
