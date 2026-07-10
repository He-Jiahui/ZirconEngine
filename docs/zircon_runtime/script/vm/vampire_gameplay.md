---
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/assets/shaders/vampire_actor/vampire_actor_base.wgsl
  - examples/vampire/assets/shaders/vampire_effect/vampire_effect_base.wgsl
  - examples/vampire/assets/shaders/vampire_forest/vampire_forest_base.wgsl
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/data/balance.toml
implementation_files:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/assets/shaders/vampire_actor/vampire_actor_base.wgsl
  - examples/vampire/assets/shaders/vampire_effect/vampire_effect_base.wgsl
  - examples/vampire/assets/shaders/vampire_forest/vampire_forest_base.wgsl
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/data/balance.toml
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
tests:
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_session_menu --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib gameplay_host_component_string_reads_string_dynamic_state --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_example_scene_extracts_playable_third_person_meshes --message-format short --color never
  - cargo test -p zircon_runtime --lib render_frame_extract_collects_world_hud_health_bars_as_scene_particles --message-format short --color never
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_reports_runtime_fps_and_render_work --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, ZR_VAMPIRE_CAPTURE_PNG, ZR_VAMPIRE_CAPTURE_WIDTH=640, and ZR_VAMPIRE_CAPTURE_HEIGHT=360 set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR and ZR_VAMPIRE_CAPTURE_PNG=examples/vampire/screenshots/vampire-runtime-point-lights-640.png
  - cargo test -p zircon_runtime --lib vampire_project_session_starts_paused_until_start_button_click --features backend-zr-vm --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR and PATH: passed 2026-06-16
  - cargo test -p zircon_runtime --lib vampire_project_session_game_over_menu_retries_to_playing --features backend-zr-vm --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR and PATH: passed 2026-06-16
  - D:\cargo-targets\zircon-runtime-06-real-backend-0616\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR and PATH: passed 2026-06-16 after EV100 scene exposure migration
  - cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_world_hud_bars --features backend-zr-vm --target-dir D:\cargo-targets\zircon-vampire-menu-vm-0611 -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, PATH, and ZR_VAMPIRE_CAPTURE_PNG=examples/vampire/screenshots/vampire-runtime-ground-fixed-640.png: pending current validation stage
  - target\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe render_product_pbr_submit_reports_material_fallback_and_light_stats --test-threads=1
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - cargo build -q -p zircon_app --bin zircon_runtime --features backend-zr-vm
doc_type: module-detail
---

# Vampire Gameplay

## Purpose

`examples/vampire/scripts/vampire_game/main.zr` is the project-local ZR script implementation behind the vampire example. It proves the current standalone runtime can host a compact third-person 3D roguelite loop with keyboard movement, automatic attacks, navmesh chase movement, world-space HUD data, and renderable effects through the generic `zr.zircon.gameplay` host module.

The example is intentionally placed under `examples/vampire` as a project asset directory. Scripts, scene files, baked navigation, materials, shaders, and model bindings are loaded through the same project manifest path used by the runtime executable.

## Player Loop

The player starts in a modal Start Game state. On the first player `onStart`, the script writes `gameplay.menu_state.state = "start"` and `vampire.run_state = "start_menu"`; the dynamic runtime extracts the menu overlay and writes `gameplay.control_state = "start_game"` when the button is clicked. The next player `onUpdate` consumes that string command, resets player state, clears the menu, and enters `vampire.run_state = "playing"`.

During play, the player uses WASD movement on the ground plane. The third-person camera follows entity `2` from a fixed above-and-behind offset, and the authored blood aura light follows the player through the same script callback path. The current script keeps the runtime path conservative: it drives movement, animation booleans, automatic damage, particles, and health bars directly from exported `onStart` and `onUpdate` functions.

Automatic attack logic targets nearby enemies and emits a short blood-bolt particle payload through `render.particle_sprites`. The authored balance file keeps the intended tuning for the full slice:

- player max HP: 120
- base move speed: 5.2 world units per second in balance data
- Blood Bolt range: 9.0, cooldown: 0.65 seconds, damage: 14
- skeleton HP/damage/speed: 24 / 8 / 3.3
- zombie HP/damage/speed: 46 / 12 / 2.35
- ghost HP/damage/speed: 18 / 6 / 3.9
- boss HP/damage/speed: 450 / 20 / 2.15

## Enemy Loop

The scene starts with authored GLB enemy actors instead of capsule stand-ins. Current archetypes are `skeleton`, `zombie`, and `ghost`; the script also keeps a boss path for `role = "boss"` bindings. One skeleton, one zombie, and one ghost keep `script.bindings.enabled = true` and run the real VM chase/combat loop. Duplicate enemies remain visible scene actors with disabled script bindings so the frame still has visual density without paying VM callbacks for every repeated model.

Each script-driven actor carries behavior metadata and health data in `script.bindings`. Health bars are written through `render.world_hud_bars` so render extraction can turn them into camera-facing pips above the model without needing a separate mesh per bar.

Enemies use baked navigation movement when chasing. The host writes navmesh agent data and asks the navigation runtime to tick agents toward the player. If no path can be resolved, enemies remain blocked rather than crossing authored obstacles through direct translation.

The enemy and boss callbacks check `game_is_playing(player)` before movement or contact damage. While the start or game-over menu is active they idle and leave navigation state alone. A lethal contact writes `gameplay.menu_state.state = "game_over"` plus `vampire.run_state = "game_over"`; clicking Retry writes `gameplay.control_state = "retry_game"`, which the player script consumes through the same `start_game` reset path used by the start button. The script uses the host-side predicates `gameplay.entity_exists(...)` and `gameplay.script_number_at_most(...)` for entity/numeric checks because the current real ZR VM path is stable when these comparisons happen inside Rust host functions, while direct comparisons of host-returned entity/numeric values in ZR script remain a boundary risk.

## World Operations

`gameplay_host.rs` is the ECS mutation owner for this gameplay slice. It exposes transform movement, camera follow, light follow, animation booleans, damage, particle sprites, script-binding queries, and `nav_move_towards_entity` to ZR scripts. The script remains project-owned and uses these generic host calls instead of a vampire-specific Rust delegate.

Keeping world mutation behind the host module prevents the ZR script from reaching into runtime internals. Tests inspect the expected host markers and world dynamic components without depending on private local variables in the script.

## Scene Lighting

The vampire scene now uses authored point-light entities as renderer inputs instead of relying on hard-coded shader light positions. The fire baskets and lantern meshes remain at ground placement, while separate light-only entities place the actual point lights at flame/lantern height. Render extraction emits those point lights through `LightingExtract`, the basic renderer packs the first fixed point-light slots into `SceneUniform`, and the vampire WGSL materials sample `scene.point_light_position_range` plus `scene.point_light_color_intensity` to produce local warm, ghost-blue, and orchid-green falloff.

This keeps the example aligned with the runtime renderer contract: scene-authored point lights affect forward/deferred shading and report ready in render stats up to the current uniform cap. Additional point lights still exist as authored scene data, but over-cap lights are diagnostic degraded until the clustered/Forward+ light-list path lands.

The ground shader now also carries a floor-light guard for terrain readability. Jungle ground and the darker graveyard floor path clamp their minimum diffuse lighting through `vampire_ground_light_floor`, while the forest detail color and fog tint are lifted enough that the baked terrain no longer falls into black under the moon/fog mix. The same WGSL logic is synchronized across the default PBR shader and the vampire actor/effect/forest base shader copies so generated variants do not regress to the old dark floor threshold.

The scene camera exposure now uses the runtime EV100 contract directly. `examples/vampire/assets/scenes/main.scene.toml` sets `exposure_ev100 = 9.2`; the old near-zero value belonged to the pre-exposure-multiplier interpretation and over-brightened the PP-M3 exposure resolve path into an all-white offscreen capture. The 2026-06-16 real-backend HUD capture passes after this asset migration, with `particle-render`, post-process output transfer, and FXAA all active in the graph.

## Current Limitation

The vampire script package compiles and loads as `zr_vm:project`, and the real VM export path can execute the exported callbacks. Local validation found these ZR script constraints to keep in mind:

- Direct host calls from exported callbacks are stable in the current build; routing those calls through local script helper functions can still trigger a native `zr_vm_core.dll` access violation.
- Runtime multiplication expressions such as `dx * speed * dt` can currently fail with `MUL_FLOAT requires numeric operands`; the current script avoids those expressions and passes simple numeric arguments into host calls.
- Direct branch comparisons against host-returned entity/numeric values are still avoided in favor of host-side predicate functions such as `entity_exists` and `script_number_at_most`.
- The real VM hot path is intentionally trimmed to the player plus three active enemy bindings. The vampire scene sets `fixed_update = false` on its bindings because its gameplay lives in `onStart` and `onUpdate`; this avoids paying real VM calls for an empty fixed-update export. The 2026-06-11 post-skip diagnostic run reported `fps_current=60.872053031732605`, `frame_ms_current=16.427899999999998`, one submitted frame, 116 mesh draws, one UI graph pass, and zero screen-space UI commands.

The latest accepted visual evidence remains the checked-in real VM offscreen export at `examples/vampire/screenshots/vampire-runtime-point-lights-640.png`, while the 2026-06-16 direct real-backend test executable also passed `vampire_project_session_capture_frame_draws_world_hud_bars` after the EV100 exposure migration. That path validates scene-following player and enemy world HUD bars, shadow work, attack particle data, no screen-space combat HUD commands, and visible scene-authored point-light contribution. The previous accepted capture remains at `examples/vampire/screenshots/vampire-runtime-playable-current-640.png`. A 1280x720 capture with one script tick currently trips a native access violation before PNG export, so high-resolution capture remains a render/VM boundary risk rather than an accepted path.
