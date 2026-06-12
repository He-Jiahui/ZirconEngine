# Vampire Roguelite Jungle And Animation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the `examples/vampire` roguelite gameplay loop, jungle navigation content, HUD bridge, and follow-on skeletal animation playback/state-machine capability.

**Architecture:** Gameplay rules live in the vampire project script and use reusable `zircon_runtime` gameplay host calls. Runtime-owned support handles dynamic visible entity spawning, death-aware damage, HUD extraction, and later animation sampling/state-machine evaluation. The example scene remains a project asset under `examples/vampire`.

**Tech Stack:** Rust `zircon_runtime`, Zr VM script host modules, TOML scene/navmesh/assets, screen-space `UiRenderExtract`, existing render framework and scene ECS.

---

## Milestone 1: Gameplay Host And HUD Bridge

**Files:**
- Modify: `zircon_runtime/src/script/vm/gameplay_host.rs`
- Modify: `zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs`
- Modify: `zircon_runtime/src/script/vm/host/builtin_host_modules.rs`
- Modify: `zircon_runtime/src/dynamic_api/session.rs`
- Modify/Create: `zircon_runtime/src/dynamic_api/session/tests.rs`
- Update docs: `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`, `docs/zircon_runtime/dynamic_api/session.md`

- [ ] Add host functions for `damage_entity_report`, `spawn_model`, `set_hud_text`, and enough component JSON helpers to avoid vampire-only branches.
- [ ] Preserve existing `damage_entity` behavior while adding the death-aware variant that returns JSON with `hit`, `killed`, and `remaining_hp`.
- [ ] Build a dynamic-session HUD extract from the default world HUD component and submit frames with `submit_frame_extract_with_ui`.
- [ ] Extend fallback backend with the same observable gameplay outcomes so tests work without the real Zr VM dynamic library.
- [ ] Add tests that prove HUD text is rendered into the frame path and death-aware damage exposes enemy death without losing component state too early.
- [ ] Milestone testing stage: run focused `cargo test -p zircon_runtime --lib vampire_project_session -- --nocapture`, then debug and correct failures.

## Milestone 2: Vampire Gameplay Loop

**Files:**
- Modify: `examples/vampire/scripts/vampire_game/main.zr`
- Modify: `examples/vampire/assets/data/balance.toml`
- Modify: `examples/vampire/assets/scenes/main.scene.toml`
- Modify: `zircon_runtime/src/dynamic_api/session/tests.rs`
- Modify: `zircon_runtime/src/asset/tests/project/example_vampire.rs`
- Update docs: `examples/vampire/README.md`

- [ ] Add deterministic random helpers, spawn timer, game timer, boss timer, player XP/level/HP/shield/buff state, weapon inventory state, upgrade prompt state, and chest prompt state.
- [ ] Spawn enemies around the player within a configured annulus and cap alive enemies.
- [ ] Award XP and run drop rolls when `damage_entity_report` reports a kill.
- [ ] Add pickup collection for XP shards, heal, shield, attack buff, attack speed buff, and chest.
- [ ] Add three-choice upgrade prompt for HP, attack, and movement speed.
- [ ] Add chest three-choice weapon selection for Blood Bolt upgrades, Orbit Blade, Lance, and Pulse Curse.
- [ ] Add boss spawn path at 300 seconds with large HP and boss HUD flag.
- [ ] Add project/session tests that force time forward and assert spawn, kill XP, level-up prompt, pickup, buff, chest, and boss state.
- [ ] Milestone testing stage: run the vampire-focused script/session tests and the vampire manifest/import test.

## Milestone 3: Jungle Terrain, Navigation, And Visual Content

**Files:**
- Modify: `examples/vampire/assets/scenes/main.scene.toml`
- Modify: `examples/vampire/assets/navigation/main.navmesh.toml`
- Create/Modify: `examples/vampire/assets/materials/*.zmaterial`
- Create/Modify: `examples/vampire/assets/models/*.model.toml`
- Modify: `zircon_runtime/src/asset/tests/project/example_vampire.rs`
- Update docs: `examples/vampire/README.md`, `docs/zircon_runtime/graphics/tests/project_render.md`

- [ ] Replace the graveyard-first layout with a jungle clearing built from a terrain grid, corridor-like walkable spaces, and dense decoration.
- [ ] Author the navmesh polygons to match the terrain corridors and obstacle clusters.
- [ ] Keep enemies and player on walkable positions and update tests to assert multi-polygon navmesh content.
- [ ] Verify offscreen render has visible terrain/decorations and the live window screenshot shows jungle content.
- [ ] Milestone testing stage: run project render/export tests and capture a current screenshot under `examples/vampire/screenshots/`.

## Milestone 4: Animation Import, State Machine, And Skeletal Playback

**Files:**
- Modify: `zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs`
- Modify: `zircon_runtime/src/asset/importer/ingest/import_gltf.rs`
- Modify: `zircon_runtime/src/asset/assets/animation.rs`
- Modify: `zircon_runtime/src/scene/components/scene.rs`
- Modify/Create focused animation runtime modules under `zircon_runtime/src/scene/` or `zircon_runtime/src/animation/` according to the existing module layout at implementation time.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs`
- Modify: `examples/vampire/assets/scenes/main.scene.toml`
- Modify: `zircon_runtime/src/asset/tests/assets/gltf_importer.rs`
- Add/Modify animation runtime tests.
- Update docs under `docs/zircon_runtime/asset/`, `docs/zircon_runtime/scene/`, and `examples/vampire/README.md`.

- [ ] Import glTF animation channels into clip tracks instead of placeholder data.
- [ ] Add clip sampling for translation/rotation/scale tracks.
- [ ] Add state-machine evaluation for active state and clip selection from movement/action parameters.
- [ ] Connect sampled joint transforms to the existing skinned draw palette path.
- [ ] Replace vampire scale-only cues with animation state parameters.
- [ ] Add tests for clip import, clip sampling, state transition, and palette output.
- [ ] Milestone testing stage: run affected animation/import/render tests, then run the vampire render/session tests.

## Final Validation

- [ ] Run `cargo fmt --check` on touched Rust files.
- [ ] Run focused `cargo test -p zircon_runtime --lib` tests for vampire project/session, UI bridge, navigation import, and animation import/playback.
- [ ] Run the vampire runtime window with the real project and capture a screenshot.
- [ ] Report exact commands, pass/fail status, remaining risks, and screenshot path.
