# Zircon Vampire Roguelite

This is a playable Zircon Runtime game slice set in a jungle-ruin clearing. It combines generated jungle terrain/foliage assets with a CC0 Kenney GLB prop and character subset.

- Menu: the sample opens on a Start Game overlay. Click Start Game to enter the run; a lethal hit opens Game Over and Retry.
- Controls: WASD movement after Start Game.
- Camera: third-person follow camera using a fixed offset.
- Combat: Blood Bolt auto-targets nearby enemies and emits a short blood particle payload through the runtime particle host call.
- Progression: balance data defines the intended XP, upgrade, and wave curve for the larger survival loop.
- Boss loop: the script keeps a `role = "boss"` path with higher HP and slower navmesh chase movement.
- Navigation: enemies are authored to use baked-scene navigation data over a multi-polygon jungle clearing navmesh.
- Scripting: `scripts/vampire_game` is a ZrVM package with entity lifecycle exports.
- Assets: generated jungle terrain, broadleaf, fern-cluster and grass static-batch model TOMLs, `assets/textures/jungle_ground_albedo.png`, Kenney GLB characters/ruin props with imported primitive/material bindings, actor material color-map textures, a compound `.zshader` package, and `navigation/main.navmesh.toml`.
- Look: uneven low-poly jungle terrain with a real terrain albedo texture, mud trails, shader-detailed forest ground/foliage, billboard grass static batches, broadleaf canopy props, fern banks, root walls, moss rocks, scene-authored relic/brazier/lantern/orchid/firefly point lights, blue moon key light, camera post-processing, a global post-process volume, fog, vignette, grain, bloom, and ACES/filmic tonemapping.
- AI: enemies use `assets/data/enemy_behavior_tree.toml` as the authored Selector/Sequence behavior tree contract. Scene script bindings still use the compatibility id `behavior_tree = "graveyard_enemy_bt"`, and the script mirrors that tree at runtime with attack, chase, and patrol branches.
- Action state: `vampire.action_state` uses `0 = idle/patrol`, `1 = run/chase`, and `2 = attack`; `vampire.behavior_node` stores the active branch code. Runtime-visible state feedback uses actor facing, scale pose changes, and a player-following blood aura light.
- Runtime budget: the real VM hot path keeps the player plus one skeleton, one zombie, and one ghost active. Duplicate visible enemies remain in the scene with disabled script bindings so the frame keeps model density without paying extra VM callbacks.
- Screenshot exports: current validation can write `screenshots/vampire-runtime-start-menu-640.png`, `screenshots/vampire-runtime-ground-fixed-640.png`, and `screenshots/vampire-runtime-game-over-640.png`.

Run from the repository root after building with the first-party runtime plugin features:

```powershell
cargo run -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --bin zircon_runtime -- --project E:\Git\ZirconEngine\examples\vampire
```

The scene uses generated project-local jungle assets plus the checked-in CC0 Kenney Graveyard Kit subset under `assets/models/kenney_graveyard`. GLB import is handled by the runtime built-in glTF importer, and the project still selects the first-party `gltf_importer` plugin when the host exposes that catalog path.

Current runtime acceptance:

- The project loads as a `zr_vm:project` script package and the real VM offscreen runtime path renders from `--project E:\Git\ZirconEngine\examples\vampire`.
- The run starts paused behind the Start Game overlay, then enters play through the dynamic menu button path.
- Fatal contact opens the Game Over overlay, and Retry resets the player to the clearing with full HP.
- The scene imports real GLB characters/props and generated jungle terrain/foliage assets.
- The scene renders visible mesh pixels from the project camera instead of a fallback or blank frame.
- Camera-visible dressing includes uneven terrain, mud trails, broadleaf canopy, fern banks, billboard grass batches, root walls, relic flames, orchid lights, fireflies, crypt ruins, and old stone props so the first frame reads as an authored jungle ruin rather than a sparse test arena.
- `grass_billboard_static_batch.model.toml` merges many grass cards into one primitive, and the scene places six `Static Grass Batch ...` entities. Runtime frame extraction groups those six static entities into one `GeometryExtract::static_batches` entry keyed by model, mesh, material, and render layer; GPU draw-call merging can consume that DTO in a later renderer pass.
- The jungle ground material binds `res://textures/jungle_ground_albedo.png`, and the navmesh has multiple walkable clearing/corridor polygons with authored height variation.
- GLB entities bind imported primitive materials, and player/skeleton/zombie/ghost override materials now bind their source GLB color-map textures. The example shader also has actor-detail plus forest ground/foliage/grass paths and a ground-light floor so characters and terrain no longer render as flat pure-color silhouettes or black ground.
- Current glTF animation channel import is still placeholder-only, so the example does not claim full skeletal clip playback. The visible runtime action states are script-driven transform pose cues until real clip import lands.
- WASD keys are read by `gameplay.key_pressed` and drive player translation/facing from `onUpdate`.
- WASD is mapped for the current third-person view: `W` moves toward negative Z, `S` toward positive Z, `A` toward positive X, and `D` toward negative X.
- Automatic attack uses `nearest_by_script_property`, `damage_entity`, `set_animation_bool`, and `set_particle_sprites` from the generic gameplay host API.
- Enemy chase uses `nav_move_towards_entity`, which writes a navmesh agent and ticks the navigation manager for the selected actor.
- Player and enemy health bars are scene-following world HUD bars emitted through `render.world_hud_bars`; the runtime frame path reports zero screen-space combat HUD commands.
- `vampire_example_manifest_scene_and_scripts_are_importable` validates actor material texture slots, shader actor/forest detail support, scene material assignment, static grass batches, multi-primitive GLB enemies, and enemy behavior-tree binding. `vampire_example_scene_extracts_playable_third_person_meshes` validates the runtime extract contains the grass renderables and their static batch.
- Imported materials use the example default PBR shader alias `res://shaders/default_pbr`, backed by `assets/shaders/default_pbr/default_pbr.zshader`.
- The latest accepted real VM offscreen visual exports for this slice are `screenshots/vampire-runtime-start-menu-640.png`, `screenshots/vampire-runtime-ground-fixed-640.png`, and `screenshots/vampire-runtime-game-over-640.png`. The previous point-light baseline remains `screenshots/vampire-runtime-point-lights-640.png`.
- The latest real VM performance diagnostic reported `fps_current=60.872053031732605`, `frame_ms_current=16.427899999999998`, one submitted frame, 116 mesh draws, and zero screen-space UI commands after the hot-path trimming and fixed-update phase skip.
- The current real VM script subset avoids local script helper-call indirection and runtime multiplication expressions; those paths exposed VM boundary issues during validation.
- A 1280x720 capture with one script tick currently exits with Windows status `-1073741819` before writing the PNG, so high-resolution capture remains a known render/VM boundary risk.

Balance data lives in `assets/data/balance.toml`. The current target remains a 10-minute survival loop with player HP 120, enemy waves, XP/level progression, temporary buffs, rare chest weapon upgrades, and five-minute boss pressure; the present real-script slice focuses on Start/Retry flow, WASD movement, third-person camera, automatic attack, navmesh chase, particles, world-space health bars, and a readable jungle floor.
