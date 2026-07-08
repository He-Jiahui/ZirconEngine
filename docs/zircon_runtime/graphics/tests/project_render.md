---
related_code:
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/materials/player_blood.zmaterial
  - examples/vampire/assets/materials/pale_bone.zmaterial
  - examples/vampire/assets/materials/ghoul_shadow.zmaterial
  - examples/vampire/assets/materials/ghost_mist.zmaterial
  - examples/vampire/assets/materials/forest_grass_billboard.zmaterial
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/assets/shaders/default_pbr/default_pbr.zshader
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/data/enemy_behavior_tree.toml
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
implementation_files:
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/materials/player_blood.zmaterial
  - examples/vampire/assets/materials/pale_bone.zmaterial
  - examples/vampire/assets/materials/ghoul_shadow.zmaterial
  - examples/vampire/assets/materials/ghost_mist.zmaterial
  - examples/vampire/assets/materials/forest_grass_billboard.zmaterial
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/assets/shaders/default_pbr/default_pbr.zshader
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/data/enemy_behavior_tree.toml
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
plan_sources:
  - user: 2026-06-09 build and visually verify the examples/vampire third-person roguelite scene
  - docs/superpowers/plans/2026-06-09-vampire-dark-content-upgrade.md
  - docs/superpowers/specs/2026-06-09-vampire-dark-content-upgrade-design.md
  - docs/superpowers/plans/2026-06-10-vampire-forest-rendering-static-batch.md
  - docs/superpowers/specs/2026-06-10-vampire-forest-rendering-static-batch-design.md
tests:
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable -- --nocapture
  - cargo test -p zircon_runtime --lib vampire_example_scene_extracts_playable_third_person_meshes -- --nocapture
  - cargo test -p zircon_runtime --lib geometry_extract_builds_static_mesh_batches_by_resource_key -- --nocapture
  - cargo test -p zircon_runtime --lib example_vampire_scene_renders_visible_mesh_pixels -- --nocapture
  - cargo test -p zircon_runtime --lib export_example_vampire_scene_png -- --ignored --nocapture
  - cargo test -p zircon_runtime --lib gameplay_pose_exports_update_entity_transform -- --nocapture
  - cargo test -p zircon_runtime --lib vampire_project_session -- --nocapture
  - cargo build -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --bin zircon_runtime
doc_type: testing-guide
---

# Project Render Tests

`zircon_runtime/src/graphics/tests/project_render.rs` owns project-level render
regressions that need the asset pipeline, scene loader, material binding, and
renderer to work together. The vampire example now uses this file as its visual
acceptance harness because the runtime window path can still fail independently
of scene rendering when swapchain/window sizing regresses.

## Vampire Visible-Pixel Regression

`example_vampire_scene_renders_visible_mesh_pixels` opens
`examples/vampire`, imports the first-wave runtime assets, renders the main
scene through the offscreen project renderer, and counts non-background pixels.
The test is intentionally broad: it catches missing scene entities, broken mesh
primitive bindings, shader compile failures, failed GLB texture binding, and
blank camera output.

`vampire_example_manifest_scene_and_scripts_are_importable` is the cheaper
project acceptance gate. It verifies that player, skeleton, zombie, and ghost
materials bind GLB color-map textures, that the default PBR shader still
contains the vampire actor-detail and forest-detail paths, that scene
mesh/primitive bindings use those actor materials, that six static grass-batch
entities use `grass_billboard_static_batch.model.toml` and
`forest_grass_billboard.zmaterial`, that the frame extract collapses those
grass entities into one runtime static batch, that the first frame has camera-visible
lanterns, tombstones, braziers, broken fences, forest grass, and local lights,
and that enemy script bindings carry the `graveyard_enemy_bt` behavior-tree id.

The scene-side acceptance target is not just "a frame exists". The vampire
scene is expected to contain a dense jungle-graveyard layout with a player
character, multi-primitive GLB enemy actors, environmental props, light volumes,
post-processing settings, billboard grass static batches, and GLB primitive
material references. If the test starts passing with only a flat floor, simple
capsule enemies, or pure debug colors, the acceptance threshold should be
tightened rather than treated as a content success.

## Manual Screenshot Export

`export_example_vampire_scene_png` is an ignored test for manual inspection. It
uses the same rendering setup as the visible-pixel regression, then writes:

`examples/vampire/screenshots/vampire-runtime-offscreen.png`

This screenshot is the reliable visual artifact for the current vampire example
because it validates the renderer and content without depending on OS window
presentation. A separate runtime-window capture may still be useful for input
and swapchain validation, but it should not replace this offscreen export until
the window surface is known to create a non-zero client area and present frames
normally.

The latest Windows runtime-window capture for manual review is:

`examples/vampire/screenshots/vampire-runtime-window-current.png`

That capture validates the standalone executable path and swapchain presentation
on top of the offscreen render acceptance.

## Current Coverage

The vampire project render coverage verifies:

- the example project can be opened by the runtime asset manager;
- the GLB import artifacts can feed scene mesh primitive references;
- the default PBR shader binds material textures and samples model UVs;
- actor override materials keep visible imported color-map detail instead of
  replacing characters with flat pure-color materials;
- dense camera-visible jungle-graveyard dressing includes lanterns, tombstones,
  braziers, broken fences, crypts, graves, crosses, and static grass batches;
- forest shader markers cover ground, foliage, and grass details while
  preserving the default PBR material layout;
- grass uses both asset-level and runtime extract-level static batching: one
  grass model merges many billboard cards, and the six static scene entities are
  grouped into one `GeometryExtract::static_batches` entry for renderer-side use;
- the graveyard scene renders visible geometry from the gameplay camera;
- the vampire script host can drive transform-level pose cues for action states;
- enemy scene bindings declare the authored behavior-tree contract id;
- project-session gameplay covers WASD movement, automatic Blood Bolt damage
  with attack action-state feedback, and enemy behavior-tree chase movement;
- the manual screenshot path produces a PNG for visual review.

It does not yet verify long-run wave spawning balance or skeletal animation
playback. Current action-state evidence is transform-level facing/scale feedback
plus dynamic state components; real glTF clip playback still belongs to a future
animation importer milestone.

## 2026-07-07 Project Scenes PBR Matrix Helper Split

Status `runtime_15_production_file_budget_ui_ibl_project_owner_split_static_passed_cargo_check_offline_locked_blocked` keeps `graphics/tests/project_render/project_scenes.rs` as the product-test entry owner and moves the large PBR/HDRI matrix helper set into `graphics/tests/project_render/project_scenes/pbr_matrix.rs`. The child owns matrix dimensions, test output directories, source-cubemap environments, project writing, luma range helpers, and PBR/HDRI response assertions. The parent keeps the ignored export/product test functions so existing guard anchors and manual product filters remain stable without adding legacy aliases or facade shims.

Verification passed scoped rustfmt, standalone structure-convention `production_file_budget` 104/104, and no-default-features runtime tests offline cargo check with warnings only. The locked Cargo gate is blocked by current non-slice `Cargo.lock` drift.
