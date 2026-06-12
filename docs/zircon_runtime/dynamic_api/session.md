---
related_code:
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/script_manifest.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/scripts/vampire_game/main.zr
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
implementation_files:
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/script_manifest.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/scripts/vampire_game/main.zr
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Zircon Runtime 独立 3D 游戏能力与 Vampire 示例计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - user: 2026-06-10 vampire screen-space HUD, buff particles, shader lighting, and no model health bars
  - user: 2026-06-11 vampire readable ground, start menu, game-over menu, Start/Retry buttons, and screenshot validation
  - user: 2026-06-12 remove runtime Vampire fallback backend
tests:
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/session/*.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/tests/*.rs
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --message-format short
  - cargo check -p zircon_runtime --lib --message-format short --color never with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo check -p zircon_runtime --lib: passed 2026-06-09
  - cargo build -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --bin zircon_runtime with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
  - target\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09; startup no longer reports vm backend not registered
  - cargo test -p zircon_runtime --lib dynamic_api::session::input_events::tests::keyboard_logical_key_maps_wasd_runtime_key_codes_for_gameplay_scripts -- --exact --nocapture: validates WASD runtime key-code mapping for project gameplay scripts
  - cargo test -p zircon_runtime --lib vampire_project_session_w_key_moves_player_before_input_clear --locked --jobs 1 --message-format short --color never -- --nocapture: passed 2026-06-09; validates project-session W input is consumed before per-frame input clearing and moves the vampire player
  - cargo test -p zircon_runtime --lib vampire_project_session -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates W movement, automatic Blood Bolt damage, player attack action-state writes, and enemy behavior-tree chase state in the standalone project session path
  - cargo test -p zircon_runtime --lib vampire_project_session_writes_hud_and_spawns_dynamic_enemy --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates runtime HUD component text and dynamic enemy spawn in the standalone project-session path
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 4 project-session tests passed, covering WASD, Blood Bolt damage, nav chase, HUD text, and dynamic enemy spawning
  - cargo test -p zircon_runtime --lib asset::tests::project::example_vampire::vampire_example_scene_extracts_playable_third_person_meshes -- --exact --nocapture: validates vampire scene import, mesh extraction, and third-person camera direction
  - target\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire captured examples\vampire\screenshots\vampire-runtime-playable.png on 2026-06-09; WASD movement is covered by the project-session input test because the follow camera can keep the player visually centered
  - cargo test -p zircon_runtime --lib runtime_session_error_preserves_step_when_inner_error_is_empty --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo build -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09 after built-in GLB/image/text importer promotion
  - D:\cargo-targets\zircon-vampire-app\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: remained running for 18 seconds on 2026-06-09; stdout/stderr empty, runtime log file empty, examples/vampire zmeta scan found no preview_state error or plugin_required GLB/image importers
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 6 project-session tests passed, covering WASD, Blood Bolt damage, nav chase, HUD text, dynamic enemy spawning, frame HUD capture, and extended-runtime HUD retention
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-capture-current.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=140 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 and exported a 1280x720 late-frame PNG
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates generated jungle models, terrain albedo texture binding/import, multi-polygon height-varying jungle navmesh, scripts, shaders, and GLB records
  - cargo test -p zircon_runtime --lib vampire_example_scene_extracts_playable_third_person_meshes --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates third-person camera and jungle scene mesh extraction
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 10 project-session tests passed, covering WASD, auto attacks, nav chase, HUD, dynamic spawning, XP/level choice, pickups/buffs, chest weapon choice, boss spawn, and screenshot HUD capture
  - cargo check -p zircon_runtime --lib --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after the graphical HUD, terrain-backed jungle, and screen-space health HUD update; existing zircon_runtime warnings only
  - cargo test -p zircon_runtime --lib runtime_session_hud_extract_builds_graphical_vampire_bars_from_text --locked --message-format short -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates graphical vampire HUD bars, icon slots, and text payloads
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after final terrain-backed jungle update; validates scene terrain component, TerrainAsset source, material layer, and navmesh height variation
  - cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 10 project-session tests passed after graphical screen-space HUD update
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-frame.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=60 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; exported final PNG with graphical HUD and terrain scene
  - cargo check -p zircon_runtime --lib --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after screen-space HUD, dynamic attack particles, and shader update; existing zircon_runtime warnings only
  - cargo test -p zircon_runtime --lib runtime_session_hud_extract_builds_graphical_vampire_bars_from_text --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates richer graphical HUD command and color payloads
  - cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 11 project-session tests passed, including no model health bars, base/buffed particles, HUD, choices, pickups, boss spawn, and frame capture
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-hud-particles-shadow.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=90 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; exported runtime screenshot with screen-space HUD, shadow stats, and attack-particle data
  - cargo build -p zircon_app --bin zircon_runtime --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; built standalone runtime executable with existing warnings only
  - D:\cargo-targets\zircon-vampire-app\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire: remained running for 8 seconds on 2026-06-10 with empty stdout/stderr, then was stopped by the smoke test
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/session/menu.rs zircon_runtime/src/dynamic_api/session/tests.rs zircon_runtime/src/script/vm/gameplay_host.rs zircon_runtime/src/script/vm/gameplay_host/tests.rs zircon_runtime/src/asset/tests/project/example_vampire.rs: pending current validation stage for the 2026-06-11 start/game-over menu and readable-ground update
  - cargo test -p zircon_runtime --lib runtime_session_menu --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib gameplay_host_component_string_reads_string_dynamic_state --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_project_session_starts_paused_until_start_button_click --features zr-vm-real-backend --target-dir D:\cargo-targets\zircon-vampire-menu-vm-0611 -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR and PATH set to the local ZrVM MSVC build: pending current validation stage; optionally exports ZR_VAMPIRE_START_MENU_CAPTURE_PNG
  - cargo test -p zircon_runtime --lib vampire_project_session_game_over_menu_retries_to_playing --features zr-vm-real-backend --target-dir D:\cargo-targets\zircon-vampire-menu-vm-0611 -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR and PATH set to the local ZrVM MSVC build: pending current validation stage; optionally exports ZR_VAMPIRE_GAME_OVER_CAPTURE_PNG
  - cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_world_hud_bars --features zr-vm-real-backend --target-dir D:\cargo-targets\zircon-vampire-menu-vm-0611 -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, PATH, ZR_VAMPIRE_CAPTURE_PNG, and 640x360 capture settings: pending current validation stage
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs zircon_runtime/src/dynamic_api/tests/support.rs: passed 2026-06-12 after optional render-bridge and headless/minimal lifecycle update
  - git diff --check -- zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs zircon_runtime/src/dynamic_api/tests/support.rs docs/zircon_runtime/dynamic_api/session.md docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md docs/plans/zircon_runtime/runtime/index.md .codex/sessions/20260612-0847-runtime-architecture-implementation.md: passed 2026-06-12 with LF-to-CRLF warnings only
  - conflict-marker/trailing-whitespace scan over the headless lifecycle code, runtime 10 plan docs, this module doc, and the active session note: passed 2026-06-12
  - source-token guard for optional RuntimeRenderBridge, uses_render_bridge(), render-bridge skip logging, and default headless test session helper: passed 2026-06-12
  - cargo test -p zircon_runtime --lib destroy_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-headless-0612 --message-format short --color never -- --nocapture: timed out after 904s on 2026-06-12 during Windows test-target compilation while another zircon_runtime cargo lane was active; the orphaned validation process was stopped and no Cargo pass is claimed
  - rustfmt --edition 2021 --check zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs: passed 2026-06-12 after removing runtime project fallback backend
  - git diff --check -- zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs docs/zircon_runtime/script/vm/zr_vm_host_reflection.md docs/zircon_runtime/dynamic_api/session.md: passed 2026-06-12 with LF-to-CRLF warnings only
  - cargo test -p zircon_runtime --lib discovery_rejects_zr_vm_project_fallback_backend --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-no-fallback-0612 --message-format short -- --nocapture --test-threads=1: timed out after 604s on 2026-06-12 during Windows test-target compilation; orphaned cargo/rustc processes were stopped and no Cargo pass is claimed
doc_type: module-detail
---

# Dynamic Runtime Session

`zircon_runtime::dynamic_api::session` owns the runtime session state behind the exported `zircon_runtime_interface` C ABI. The public function table remains in `dynamic_api::exports`; session code owns handle validation, session registry access, event dispatch into runtime managers, frame capture/presentation, profile control, and host request draining.

The session module is intentionally private to `zircon_runtime`. Its job is to adapt versioned ABI payloads into existing runtime facilities without turning the dynamic library boundary into a second runtime architecture.

## Owner Split

- `session.rs` keeps the FFI entry functions, session registry, `RuntimeDynamicSessionProfile`, and `RuntimeDynamicSession` lifecycle/orchestration.
- `session/project.rs` adapts the optional ABI project-manifest byte slice into a runtime project root, loads `zircon-project.toml`, discovers startup ZrVM packages, and loads the manifest default scene through the existing scene/project asset path.
- `session/status.rs` owns ABI `ZrStatus` construction for unsupported version, invalid argument, not found, and generic dynamic API errors.
- `session/host_requests.rs` converts neutral runtime input-manager host requests into ABI host request payloads.
- `session/input_events.rs` maps ABI numeric input/window/gamepad/IME constants into `core::framework::input` DTOs, including the ASCII-style WASD and digit-key codes emitted by the standalone app's physical-key converter for gameplay scripts and choice prompts.
- `session/preview.rs` owns fallback frame and accessibility preview payloads used when the dynamic preview cannot extract a full UI surface.
- `session/hud.rs` extracts runtime HUD UI from gameplay dynamic components such as `gameplay.hud_text` and passes it through the normal frame presentation path. Generic text still has a fallback path, while the vampire sample HUD parser converts structured HP/XP/time/weapon/buff text into a graphical screen-space panel with bars, icon slots, and prompt rows. Combat health bars for the sample must remain in this HUD layer instead of reappearing as scene mesh/cube entities.
- `session/menu.rs` extracts the runtime gameplay menu overlay from `gameplay.menu_state` and handles pointer hit-testing for its single command button. It writes the selected command back through `gameplay.control_state`, keeping Start/Retry interaction in the same dynamic-component channel the project script can consume on the next tick.
- `session/tests.rs` owns focused tests for the private session orchestrator, including the vampire project-session input, combat, action-state, enemy behavior-tree, world HUD, start menu, game-over retry menu, capture export, and render diagnostic acceptance checks.
- `tests/` mirrors the same owner split for the exported API table, profile control, viewport/frame validation, session lifecycle, host requests, accessibility, and input-event rejection paths.
- `tests/structure.rs` keeps that mirror executable by rejecting a recreated `tests.rs`, missing owner modules, non-navigational `mod.rs` content, and owner files that grow past the split threshold.

This keeps the FFI boundary file below the large-file warning line while preserving the exported `ZrRuntimeApiV1` shape.

## Session Lifecycle Failure Contract

Session lifecycle failures are part of the exported ABI contract, not private implementation details. `destroy_session` accepts each live handle once, then reports `NotFound` with `runtime session not found` after registry removal. `ZrRuntimeSessionHandle::invalid()` is rejected before registry lookup with `InvalidArgument` and `invalid runtime session handle`.

`destroy_session_reports_explicit_not_found_for_missing_nonzero_handle`, `destroy_session_removes_registry_entry_so_destroyed_handles_become_missing`, `session_destroy_reports_explicit_not_found_after_headless_destroy`, `all_session_entry_points_reject_invalid_handle`, `destroyed_headless_session_entry_points_reject_old_handle`, and `missing_session_entry_points_reject_nonzero_handle` lock the handle-taking `ZrRuntimeApiV1` entry points to that contract. The dynamic entry-point coverage intentionally uses otherwise valid event, frame, viewport, profile, and host-request arguments so the tests reach session validation instead of version, viewport, or payload preflight branches.

`minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation. They still register and activate runtime modules, install scene hooks, create or load a level, dispatch input, tick time, and drain host requests, but frame capture returns an empty encoded frame and surface bind/present operations are no-ops. This keeps lifecycle and ABI validation independent from WGPU device limits while preserving the rendered `runtime`/`editor`/`dev` profiles.

## Boundary Rules

The dynamic session may:

- validate ABI versions and handles before touching runtime state;
- adapt host ABI values into runtime framework DTOs;
- resolve runtime managers through the existing `CoreRuntime` handle path;
- run the runtime tick, optional render bridge, default level, project default-scene load, script scene hook installation, and camera-controller preview path.
- load startup script packages declared by `ProjectManifest.scripts` once the Script module and selected backend are active.

The dynamic session must not:

- duplicate module assembly rules owned by `zircon_runtime::builtin::runtime_modules`;
- expose new public Rust API from the dynamic ABI internals;
- encode editor authoring state as runtime session persistence;
- bypass `core::framework` DTOs with ad hoc dynamic-only event models;
- grow conversion, status, preview, or host-request helpers back into `session.rs`.
- add new dynamic API assertions back into a monolithic `tests.rs`; new coverage belongs in the matching `tests/<owner>.rs` module.

## Project Runtime Entry

`zircon_app` now accepts `--project <path>` and `--project=<path>` for the `zircon_runtime` binary. The app encodes that path into the dynamic-session ABI `project_manifest` byte slice; the runtime side treats an empty slice as the old default-level behavior and a non-empty slice as a project root.

When a project root is provided, `RuntimeProjectConfig` opens `zircon-project.toml`, reads `default_scene`, and calls `scene::load_level_asset(core, project_root, default_scene)`. That path scans/imports `project_root/assets`, loads the scene asset artifact, instantiates a `LevelSystem`, and then the session ticks that level before input `begin_frame`. The same config also reads `scripts.package_roots` and `scripts.startup_packages`, discovers `plugin.toml` packages under those roots, filters to the requested startup packages when listed, and loads them through `VmPluginManager`.

Dynamic sessions install the script fixed-update and update scene hooks after module activation and before project scripts are loaded. The helper only registers missing hook ids, so first-party ZrVM language plugin hook registration can still own the same ids when that extension path is already present. This keeps project `script.bindings` active for the standalone dynamic runtime path without requiring the dynamic library to depend directly on external `zircon_plugins` crates.

Standalone keyboard events originate in `zircon_app::entry::runtime_entry_app::converters::keyboard`. Letter and digit keys are normalized to stable ASCII-style runtime codes before crossing the ABI; `session/input_events.rs` then preserves the logical key string so `gameplay.key_pressed("W")`, `A`, `S`, `D`, and `1`/`2`/`3` work in project scripts independent of platform scan-code hashing.

`vampire_project_session_w_key_moves_player_before_input_clear` locks the tick ordering for the standalone path: the dynamic session injects the W key, runs the project level tick while input is still visible to gameplay, and only then clears per-frame input state. This prevents the app ABI path from accepting keyboard events while starving project scripts of the same input during gameplay updates.

`vampire_project_session_auto_blood_bolt_damages_nearest_enemy` locks the combat path for the same standalone entry: one project-session tick must let the vampire auto-target the nearest enemy, reduce that enemy's scripted HP, write `vampire.action_state = 2`, and author `render.particle_sprites` with the base `blood_bolt` attack style. This path must execute through the authored ZR script and generic gameplay host APIs; runtime must not satisfy the test through a Rust Vampire fallback backend.

`vampire_project_session_enemy_behavior_tree_chases_player` locks the enemy AI path: one project-session tick must evaluate the authored enemy behavior tree, put enemy 20 into chase/run state with `vampire.action_state = 1` and `vampire.behavior_node = 31`, and move the enemy closer to the player through the navigation-preferred chase path. This keeps behavior-tree binding from becoming a manifest-only claim.

`vampire_project_session_starts_paused_until_start_button_click` locks the start-menu path: the first project tick writes `gameplay.menu_state.state = "start"` and `vampire.run_state = "start_menu"`, keeps the player and enemy transforms stable, accepts a left-click over the menu button, then advances to `vampire.run_state = "playing"` and clears the menu component. This prevents the standalone sample from beginning combat before the player presses Start Game.

`vampire_project_session_game_over_menu_retries_to_playing` locks the death/retry path: a lethal enemy contact writes `gameplay.menu_state.state = "game_over"` and `vampire.run_state = "game_over"`, then a Retry click resets player HP/position and re-enters gameplay. The session test can export `ZR_VAMPIRE_GAME_OVER_CAPTURE_PNG` from the same offscreen frame path used by runtime screenshots.

`vampire_project_session_writes_world_hud_for_scene_authored_enemies` locks the visible standalone gameplay path: project ticks must keep combat health in scene-following `render.world_hud_bars`, keep authored enemies stable, and avoid recreating `health_bar_fill` or `health_bar_back` scene roles. This prevents the dynamic executable from launching a static scene while tests only prove invisible script state.

`vampire_project_session_kill_xp_opens_level_up_and_choice_updates_hp` locks the kill reward and level-up path: a killed scripted enemy must award its XP before removal, open the level-up HUD prompt, accept digit-key choice input, and apply the max-HP evolution.

`vampire_project_session_pickups_apply_heal_shield_and_timed_buffs` locks the pickup path: nearby scripted pickup entities for heal, shield, attack, and haste must be consumed by the player update and reflected in `gameplay.hud_text`.

`vampire_project_session_buffed_attack_particles_use_buff_palette` locks the buff-driven attack VFX path: after attack, haste, and shield pickups are active, the next automatic attack must write a combined `blood_flame_haste_shield` particle payload, and the authored sprites must include red/orange blood flame, cyan/green haste, and blue shield accents for the renderer.

`vampire_project_session_chest_choice_grants_weapon_upgrade_from_digit_input` locks the rare chest path: collecting a chest opens the weapon-choice prompt, digit-key input chooses a weapon, and the HUD reports the upgraded inventory.

`vampire_project_session_boss_spawn_uses_five_minute_default_with_test_override` locks the boss spawn director without making tests wait five minutes. The runtime default remains `300.0` seconds; the test injects a player script property `boss_interval_seconds` to force the same timed path immediately.

`vampire_project_session_keeps_hud_after_extended_runtime` covers the same path after a longer simulation window. The current demo now transitions fatal contact into a menu-driven game-over state instead of deleting the player entity, which keeps the third-person camera target, world HUD data, and Retry flow stable for manual screenshots and playable smoke tests.

`vampire_project_session_capture_frame_draws_world_hud_bars` captures the runtime frame after project ticks and asserts that world-space health bars, shadow work, and particle data are present while screen-space combat HUD command count stays zero. It accepts `ZR_VAMPIRE_CAPTURE_WIDTH`, `ZR_VAMPIRE_CAPTURE_HEIGHT`, `ZR_VAMPIRE_CAPTURE_TICKS`, and `ZR_VAMPIRE_CAPTURE_PNG` so manual validation can export a deterministic gameplay PNG at the requested viewport and after a longer gameplay run without adding another test-only code path.

When `current_ui_extract` sees a valid `gameplay.menu_state`, the menu overlay has priority over the gameplay HUD extract. This is intentional: Start and Game Over are modal runtime states, while the world HUD bars remain scene data. Left mouse release over the overlay button writes only the command string (`"start_game"` or `"retry_game"`) and leaves lifecycle ownership in the project script.

Dynamic API error statuses now carry the concrete session creation failure string across the ABI diagnostics slice instead of collapsing every startup error to `runtime dynamic API error`. That diagnostic payload is intentionally leaked for process lifetime because `ZrStatus` exposes only a borrowed `ZrByteSlice`; owned byte buffers remain reserved for APIs that include an explicit free callback.

`examples/vampire` is the current acceptance fixture for this project-entry path. Its manifest declares `res://scenes/main.scene.toml`, a `scripts/vampire_game` ZrVM package, project-local model/material/shader/navmesh/terrain assets, and plugin selections for rendering, animation, navigation, glTF importing, texture importing, and ZrVM language runtime. The fixture now includes generated jungle terrain/foliage models, a real project texture at `res://textures/jungle_ground_albedo.png`, a real `TerrainAsset` at `res://terrain/jungle_clearing.terrain.toml` with a ready `.zmeta`, a multi-polygon baked jungle navmesh with authored height variation, a richer `default_pbr` shader that samples material maps and folds in shadow/reflection/detail-normal terms, a ground-light floor that prevents terrain from collapsing to black under fog, script-authored dynamic attack particles, and menu state components for Start/Game Over. The scene's `Baked Jungle Terrain` entity intentionally carries both the visible mesh and the terrain component so the sample is terrain-backed instead of a decorative prop-only floor. Asset tests keep those pieces importable through the same project scan path used by the standalone runtime.

The dynamic executable links `zircon_app`, but the actual runtime session is created by `zircon_runtime.dll` through the ABI. App-side Rust plugin registration reports do not cross that dynamic boundary. The runtime therefore must keep enough default import capability for simple project startup. The default asset importer now includes built-in glTF/GLB, common image, and `.txt` text-data importers; linked plugin catalog importers can still override those matchers on static or embedded paths, but the dynamic runtime no longer depends on those app-linked registrations just to import the vampire GLB models, Kenney shared texture, or local license notes.

## Validation

`zircon_runtime/src/dynamic_api/tests/` covers the exported function table, invalid ABI and handle paths, profile-control JSON validation, frame/accessibility request validation, session creation profile handling, host request encoding, accessibility fallback behavior, and input-event rejection paths.

For architecture validation, the runtime structural audit should no longer list `zircon_runtime/src/dynamic_api/session.rs` under production large-file hotspots after this split. The audit also reports `dynamic_api_test_boundary`, which must keep the legacy `zircon_runtime/src/dynamic_api/tests.rs` absent, all owner modules declared, and oversized test owner modules at zero. That audit owner now lives in `runtime_structure_audits/dynamic_api_test_boundary.py` so the main architecture audit script remains an orchestration boundary instead of becoming another mixed large file.
