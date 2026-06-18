---
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - examples/vampire/scripts/vampire_game/main.zr
implementation_files:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
tests:
  - cargo test -p zircon_runtime --lib builtin_host_modules_register_gameplay_capabilities --message-format short --color never
  - cargo test -p zircon_runtime --lib gameplay_host_component_string_reads_string_dynamic_state --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_writes_world_hud_for_scene_authored_enemies --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_reports_runtime_fps_and_render_work --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, ZR_VAMPIRE_CAPTURE_PNG, ZR_VAMPIRE_CAPTURE_WIDTH=640, and ZR_VAMPIRE_CAPTURE_HEIGHT=360 set
  - cargo check -p zircon_runtime --lib --message-format short --color never
doc_type: module-detail
---

# Gameplay Host

## Purpose

`zircon_runtime::script::vm::gameplay_host` is the host module exposed to project scripts as `zr.zircon.gameplay`. It owns script-callable gameplay functions that need access to the active runtime world, entity ids, input state, navigation services, dynamic components, and transient render-side effects.

The module is intentionally host-side. Scripts pass primitive values such as entity ids and deltas; the host resolves those values against the active `LevelSystem` and writes typed or dynamic ECS components through runtime-owned APIs.

## Capability Surface

The gameplay host functions include entity transform reads/writes, entity existence checks, current HP writes, script-binding queries, camera/light follow helpers, animation boolean writes, particle sprite payloads, world-space HUD bars, string dynamic-component reads, and navigation movement. `component_string(entity, component_id, fallback)` reads a dynamic component only when it is a JSON string and otherwise returns the supplied fallback, which lets scripts consume runtime UI commands such as `gameplay.control_state` without parsing JSON manually. `entity_exists(entity)` and `script_number_at_most(entity, property, threshold, fallback)` keep common entity/numeric predicates on the Rust host side, avoiding current real-ZR-VM instability around comparing host-returned entity and numeric values directly inside project script branches. `nav_move_towards_entity` writes a `navigation.Component.NavMeshAgent` dynamic component for the mover, resolves the navigation service, and ticks world agents so enemies follow baked navmesh paths instead of performing raw direct translation.

The vampire example deliberately uses these generic host functions from `examples/vampire/scripts/vampire_game/main.zr`. The host does not need vampire-specific lifecycle delegates for the current script path: `onStart` and `onUpdate` query `script.bindings` properties and call the generic movement, combat, navigation, animation, particle, and HUD functions directly.

## Script State Ownership

`script_bindings.rs` owns the shared helpers that read and mutate JSON script-binding properties such as `role`, `archetype`, `hp`, `move_speed`, and `contact_damage`. These helpers keep the host API generic while still letting project scripts express gameplay roles as asset data.

Per-frame gameplay state for the current vampire slice remains intentionally small and data-oriented: health lives in script bindings, run/menu command state lives in dynamic string/JSON components, action feedback lives in dynamic components or animation parameters, and transient effects live in render dynamic components. The scene keeps only the player and three enemy bindings enabled in the real VM hot path; duplicate visible enemies can carry the same metadata with `enabled = false` so host queries, damage, and navigation skip them. Longer-lived systems such as level-up choices or timed spawners should move into explicit project data or a script-visible state component before being expanded.

## Validation Notes

The focused host registration test verifies that the built-in gameplay module exposes the reflected gameplay API. The `gameplay_host_component_string_reads_string_dynamic_state` unit test locks the string-component read helper used by the vampire menu command flow. `gameplay_host_script_property_match_and_heal_update_bindings` now also covers `entity_exists` and `script_number_at_most`, and `host_function_registry_matches_documented_ledger` keeps the ledger aligned at 52 fixed host functions / 39 gameplay callbacks. The vampire project manifest test verifies that the project script imports this host module and drives gameplay through generic markers such as `gameplay.key_pressed`, `gameplay.translate`, `gameplay.face_direction`, `gameplay.camera_follow`, `gameplay.follow_position`, `gameplay.nearest_by_script_property`, `gameplay.nav_move_towards_entity`, `gameplay.component_string`, `gameplay.entity_exists`, `gameplay.script_number_at_most`, `gameplay.damage_entity`, `gameplay.set_world_hud_bar`, `gameplay.set_animation_bool`, and `gameplay.set_particle_sprites`.

The 2026-06-11 real VM diagnostic run reported `fps_current=60.872053031732605` and `last_ui_command_count=0` for the vampire scene after disabled duplicate bindings and the fixed-update phase skip, which confirms the gameplay host path is driving scene-following HUD data and not a screen-space upper-left combat HUD.
