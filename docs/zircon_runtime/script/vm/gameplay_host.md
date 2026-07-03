---
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/error.rs
  - zircon_runtime/src/script/vm/gameplay_host/combat.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/component_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/property_animation.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - examples/vampire/scripts/vampire_game/main.zr
implementation_files:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/error.rs
  - zircon_runtime/src/script/vm/gameplay_host/combat.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/component_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/property_animation.rs
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
  - cargo test -p zircon_runtime --lib runtime_15_gameplay_host_tests_are_folder_backed --no-default-features --features core-min --locked: deferred in Runtime 15 M3 gameplay host test folder split
  - cargo test -p zircon_runtime --lib runtime_15_script_vm_gameplay_host_guard_is_child_owner --no-default-features --features core-min --locked: deferred in Runtime 15 M3 script VM gameplay host guard child-owner split
  - cargo test -p zircon_runtime --lib review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary --no-default-features --features core-min --locked: deferred while external cargo/rustc lanes are active
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

## Runtime 15 F5 gameplay host typed errors

状态：`runtime_15_gameplay_host_typed_errors_static_passed_cargo_deferred`。

Runtime 15 E1/E2/F5 的当前切片新增 `script/vm/gameplay_host/error.rs`，由 `GameplayHostError` / `GameplayHostResult` 承接 gameplay host 内部 mutation 与 navigation 错误。`combat.rs`、`lifecycle.rs`、`navigation.rs` 与 `transform.rs` 不再用内部 `Result<_, String>`、`Err(format!(...))` 或 `.map_err(|error| error.to_string())` 表达 world mutation、navigation tick、JSON serialization 或 missing-entity failure。

该 typed-error owner 保留 `SceneError`、`NavigationError` 与 `serde_json::Error` source，并用 `GameplayHostError::MissingEntity` 表达 host-local missing entity category。VM 可见边界仍返回 `ScriptHostError`，所以脚本调用方看到的 host-call 诊断形状不变；字符串化只发生在这个边界。

守卫：`review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary` 检查 `mod error;`、`GameplayHostError` / `GameplayHostResult`、四个 domain owner 的无 String-error 回流，以及 Runtime 15 子计划、runtime index、结构规范、review findings、host ledger、module-convention 和 status-output expectations 的同步锚点。验证：scoped rustfmt/static scans 通过；Cargo 因并行 cargo/rustc lane active deferred，不计通过。

## Runtime 15 M3 gameplay host test folder split

状态：`runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred`。

Runtime 15 R4.1/M3 的当前结构切片只调整 gameplay host 测试 owner，不改变 `zr.zircon.gameplay` 注册面、callback 行为或 Runtime 13 host ledger 计数。`script/vm/gameplay_host/tests.rs` 从 891 行降到 46 行，只保留共享导入、`mod combat_lifecycle;`、`mod component_state;`、`mod property_animation;`、`mod spawn_transform;` 和 `assert_vec3_close` / `assert_quat_close` helper。

9 个原测试迁入 `script/vm/gameplay_host/tests/spawn_transform.rs`、`script/vm/gameplay_host/tests/component_state.rs`、`script/vm/gameplay_host/tests/combat_lifecycle.rs` 与 `script/vm/gameplay_host/tests/property_animation.rs`；最大 child `property_animation.rs` 为 289 行，全部低于 800 行预算。新增 `structure_convention/test_file_budget/script_vm_tests.rs::runtime_15_gameplay_host_tests_are_folder_backed`，锁定父/子模块挂载、moved test 不回流、迁移测试数量、owner 行数预算，并要求 Runtime 15 计划、runtime index、结构规范、review findings、module-convention、本文档和 status-output expectations 同步该状态锚。

Runtime 15 M3 script VM gameplay host guard child-owner split status: `runtime_15_script_vm_gameplay_host_guard_child_owner_split_static_passed_cargo_deferred`.

The structure guard owner for the gameplay host test split now lives at `tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/gameplay_host.rs`, with `runtime_15_script_vm_gameplay_host_guard_is_child_owner` preventing the gameplay host checks from returning to the parent script VM test-budget guard.
