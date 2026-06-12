# Vampire Dark Content Upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade `examples/vampire` into a dark fantasy 3D roguelite slice with real CC0 assets, authored post-process mood, richer gameplay balance, and runtime/render verification.

**Architecture:** Build the lower runtime authoring support first by adding scene TOML post-process DTOs that map into the existing render extract stack. Then rebuild the vampire example as a project-local content pack using CC0 source assets, imported model/material/library records, scripted wave logic, and validation tests. Keep GLTF animation playback out of scope because the current importer only records animation placeholders.

**Tech Stack:** Rust `zircon_runtime`, TOML scene/material/model assets, project `library/` artifacts, ZrVM script package, CC0 GLTF/OBJ assets, PowerShell validation on the existing `main` checkout.

---

## File Structure

- Modify `zircon_runtime/src/asset/assets/scene.rs`: add serializable post-process scene DTOs and include them in `SceneEntityAsset` and scene overview metadata.
- Modify `zircon_runtime/src/scene/world/project_io.rs`: convert post-process scene DTOs to/from `PostProcessSettingsComponent` and `PostProcessVolumeComponent`.
- Modify `zircon_runtime/src/scene/tests/render_post_process_extract.rs`: add or extend tests proving scene-authored post-process settings enter render extracts.
- Modify `zircon_runtime/src/asset/tests/assets/scene.rs`: add TOML roundtrip coverage for post-process camera settings and global volume fields.
- Modify `docs/zircon_runtime/asset/assets/scene.md`: document the new scene fields, conversion path, and tests.
- Modify `examples/vampire/assets/scenes/main.scene.toml`: replace primitive arena composition with a crypt/graveyard scene, post-process settings, lights, and enemy archetype entities.
- Modify `examples/vampire/assets/materials/*`: add dark stone, bone, vampire, warm emissive, and fog-readable materials.
- Modify `examples/vampire/assets/models/*` and `examples/vampire/library/**`: add project-facing artifacts for selected real asset meshes; keep a few generated proxy meshes only for invisible gameplay markers if needed.
- Create/modify `examples/vampire/assets/external/**`: store CC0 source packs or extracted subsets and local license notes.
- Modify `examples/vampire/assets/data/balance.toml`: store player, weapon, enemy, XP, and wave values from the design.
- Modify `examples/vampire/scripts/vampire_game/main.zr`: add wave timers, archetype behavior, XP/stat progression, and tuned camera/attack values.
- Modify `examples/vampire/README.md` and `examples/vampire/LICENSES.md`: document controls, content sources, limitations, and validation evidence.
- Modify `zircon_runtime/src/asset/tests/project/example_vampire.rs` and `zircon_runtime/src/graphics/tests/project_render.rs`: update import/render assertions for the upgraded scene.
- Update `.codex/sessions/20260609-1551-vampire-dark-content-upgrade.md`: keep live coordination current, then retire it on completion.

## Milestone 1: Scene Post-Process Authoring Support

- Goal: Scene TOML can author camera/global post-process settings that reach `PostProcessExtract`.
- In-scope behaviors:
  - Camera entity may carry `post_process_settings`.
  - Entity may carry `post_process_volume`.
  - DTOs cover bloom, color grading, tonemap, vignette, grain, dither, chromatic aberration, fog, and minimal volume blend metadata.
  - Defaults preserve existing scenes when fields are absent.
- Dependencies: existing `PostProcessSettingsComponent`, `PostProcessVolumeComponent`, render extract volume stack, and scene world project I/O.
- Implementation slices:
  - Add `ScenePostProcessSettingsAsset`, `ScenePostProcessVolumeAsset`, and focused nested DTOs in `scene.rs`.
  - Add helper conversions between array values and `Vec3`/render settings in `project_io.rs`.
  - Insert fixed post-process components when loading scene entities.
  - Serialize fixed post-process components when converting world back to scene asset.
  - Extend scene overview with `has_post_process_settings` and `has_post_process_volume`.
  - Update module docs with related code, plan source, and tests.
- Unit-test code:
  - Add a scene TOML roundtrip test that parses a camera with post-process settings and a global volume, serializes back, and verifies fields persist.
  - Add a world/render extract test that loads a scene asset through `World::from_scene_asset` and verifies effect stack families are enabled in `extract.post_process`.
- Lightweight checks:
  - `cargo check -p zircon_runtime --lib --locked` if type errors are unclear before the testing stage.
- Testing stage:
  - Run focused tests:
    - `cargo test -p zircon_runtime --locked asset::tests::assets::scene::scene_asset_toml_roundtrip_preserves_post_process_components -- --exact --nocapture`
    - `cargo test -p zircon_runtime --locked scene::tests::render_post_process_extract::scene_asset_post_process_settings_feed_render_extract -- --exact --nocapture`
  - Debug from the lowest shared conversion helper if either fails.
- Exit evidence:
  - Both focused tests pass, and existing vampire scene still imports with absent fields or new fields.

## Milestone 2: CC0 Dark Asset Pack Integration

- Goal: `examples/vampire` contains license-clear real model/environment source assets and project-facing imported records.
- In-scope behaviors:
  - Store selected CC0 source files from KayKit, Kenney, and Quaternius under `assets/external`.
  - Prefer GLB/GLTF or OBJ files that the importer can parse today.
  - Keep the selected subset small enough for an example project.
  - Update `.zmeta` and `library/` records through the project scan/import path rather than hand-writing stale artifacts when practical.
  - Record source URL, author, license, and local file mapping in `LICENSES.md`.
- Dependencies: Milestone 1 is not required for importing assets, but the scene composition in Milestone 3 will use both.
- Implementation slices:
  - Download or extract a small subset: dungeon floor/wall/pillar/coffin/torch, graveyard tombstone/fence, vampire/player humanoid, skeleton grunt, fast monster, elite monster.
  - Normalize file placement under `assets/external/kaykit_*`, `assets/external/kenney_*`, and `assets/external/quaternius_*`.
  - Run project scan/import to generate resource records.
  - If any selected GLTF file hits importer limitations, switch that asset to OBJ or a simpler GLB from the same CC0 pack rather than broadening runtime scope.
  - Keep primitive mesh files only when needed for gameplay-only invisible markers or fallback tests.
- Unit-test code:
  - Extend `example_vampire.rs` to assert at least one imported external environment model and two imported enemy/player model sources are present.
  - Assert `LICENSES.md` contains each source pack name and CC0 license marker.
- Lightweight checks:
  - Use directory listing and project import output during asset selection.
- Testing stage:
  - Run `cargo test -p zircon_runtime --locked asset::tests::project::example_vampire::vampire_example_manifest_scene_and_scripts_are_importable -- --exact --nocapture`.
  - Debug missing records by checking `.zmeta`, `library/`, and source URI mapping.
- Exit evidence:
  - Focused vampire import test passes and `LICENSES.md` maps all external assets used in the scene.

## Milestone 3: Dark Scene, Navigation, And Gameplay Loop

- Goal: The vampire project reads as a dark fantasy game slice rather than a primitive demo.
- In-scope behaviors:
  - Scene uses real environment meshes for crypt/graveyard dressing.
  - Camera and post-process settings create a dark mood.
  - Existing WASD and auto-attack remain functional.
  - Enemies include at least three archetypes with different HP/speed/damage values.
  - `balance.toml` matches the documented initial values.
  - Baked navmesh path remains present and enemies call `nav_move_towards_entity`.
- Dependencies: Milestones 1 and 2.
- Implementation slices:
  - Replace `main.scene.toml` primitive arena with dressed static environment entities.
  - Add point/rect/spot lights for candles or braziers and reduce global exposure.
  - Add post-process settings to the follow camera and a global post-process volume if useful.
  - Expand script properties for player/enemy roles, archetypes, HP, damage, speed, XP value, and spawn timing.
  - Extend `main.zr` with wave timer, deterministic upgrade progression, cooldown/range/damage variables, and per-archetype behavior.
  - Update `balance.toml`, README, and screenshot path text.
- Unit-test code:
  - Update `vampire_example_scene_extracts_playable_third_person_meshes` to assert player, camera, multiple enemy archetypes, environment mesh count, and post-process enabled state.
  - Update or add runtime session movement test if entity IDs or camera assumptions change.
  - Update render project test to prove nonblank dark scene pixels and more than primitive mesh count.
- Lightweight checks:
  - Parse scene TOML and script package after edits.
- Testing stage:
  - Run focused import/extract tests:
    - `cargo test -p zircon_runtime --locked asset::tests::project::example_vampire -- --nocapture`
    - `cargo test -p zircon_runtime --locked graphics::tests::project_render::example_vampire_scene_renders_visible_mesh_pixels -- --exact --nocapture`
    - `cargo test -p zircon_runtime --locked dynamic_api::tests::session_lifecycle::vampire_project_session_w_key_moves_player_before_input_clear -- --exact --nocapture`
  - Debug failures from asset import to scene world conversion to render extraction to dynamic session input in that order.
- Exit evidence:
  - Focused tests pass and updated README describes the actual controls/content.

## Milestone 4: Runtime Launch And Visual Acceptance

- Goal: A real runtime launch proves the upgraded project renders and is playable enough for inspection.
- In-scope behaviors:
  - Build/run the runtime binary with required first-party plugins.
  - Launch `--project E:\Git\ZirconEngine\examples\vampire`.
  - Capture a fresh screenshot showing the dressed dark scene.
  - Record any remaining limitations honestly.
- Dependencies: Milestones 1-3.
- Implementation slices:
  - Run the runtime with rendering, texture, animation, navigation, and ZrVM plugin features.
  - Use the existing screenshot capture path/tooling from prior vampire validation.
  - Update `examples/vampire/screenshots/vampire-runtime-playable.png`.
  - Update `README.md` validation notes and `.codex/sessions` completion status.
- Testing stage:
  - Run launch command:
    - `cargo run -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --bin zircon_runtime -- --project E:\Git\ZirconEngine\examples\vampire`
  - If the binary fails before window creation, inspect the selected runtime log under `C:\Users\HeJiahui\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\`.
  - If it launches but looks blank, inspect render extraction and asset readiness records before altering shaders.
- Exit evidence:
  - Runtime exits cleanly or remains running long enough for screenshot capture.
  - Screenshot shows real dark environment meshes and player/enemy silhouettes.

## Milestone 5: Final Validation And Documentation Closeout

- Goal: The full objective is verified against source, tests, runtime behavior, and docs.
- In-scope behaviors:
  - Focused tests from prior milestones pass.
  - Module docs and example docs are updated.
  - Coordination note is retired.
  - Goal completion audit checks every explicit user requirement.
- Dependencies: Milestones 1-4.
- Testing stage:
  - Re-run focused commands from Milestones 1-4.
  - If shared runtime scene/asset APIs changed substantially, run `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` or an equivalent focused package build/test gate.
  - Avoid claiming full workspace green unless workspace validation is actually run and passes.
- Exit evidence:
  - Requirement-by-requirement audit proves: dark style, real free models, environment scene, post-process, richer gameplay, WASD, auto-attack, navmesh chase, third-person camera, balance, project under `examples/vampire`, and runtime verification.
