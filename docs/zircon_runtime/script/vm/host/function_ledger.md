---
related_code:
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/handles.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - zircon_runtime_interface/src/reflect/mod.rs
implementation_files:
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/handles.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - dev/godot/core/extension/extension_api_dump.cpp
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
  - dev/Piccolo/engine/source/runtime/core/meta/reflection/reflection.h
tests:
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_function_registry_matches_documented_ledger
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_capability_representatives_are_declared_on_registered_modules
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_function_without_required_capability_is_rejected_with_explicit_error
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_function_registry_ledger_guard_rejects_missing_entry
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs::script_held_entity_handle_reports_invalid_after_despawn
  - zircon_runtime/src/script/vm/tests.rs::script_call_table_pre_resolves_host_export_callbacks
  - zircon_runtime/src/script/vm/tests.rs::zr_vm_real_backend_uses_script_call_table_for_host_callbacks
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs::bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs::bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json (2026-06-13 Runtime 13 script_binding_boundary targeted evidence: expected_source_file_count = 10, expected_test_file_count = 2, fixed_host_function_count = 50, host_capability_count = 11, native_ecs_abi_references = [], risks = [])
  - "pending: cargo test -p zircon_runtime --lib script --locked -- --nocapture"
doc_type: module-detail
---

# ZrVM Host Function Ledger

Runtime 13 M0 treats the script host surface as a descriptor ledger, not as an ad hoc callback list. The current fixed built-in baseline is 6 host modules, 50 fixed host functions, and 2 fixed script type descriptors. The plugin bridge host module is dynamic: its module name and required baseline capability are fixed, but its callable functions are supplied by bridge method descriptors at registration time.

`HostExportRegistry::register_module(...)` is the runtime enforcement point. It validates module names, declared module capabilities, function arity, required function capabilities, callback/descriptor parity, type descriptors, and call-time capability grants. `builtin_host_module_descriptors()` plus `zircon_host_reflection_docs` are the machine-renderable source for the fixed built-in ledger.

`HostExportRegistry::script_call_table()` snapshots those validated module descriptors and callbacks into dense `ScriptCallSite` rows. The real `zr_vm` backend resolves each native host callback against that table during module registration, so runtime host calls dispatch by pre-resolved call site instead of performing module/function name lookup on every call. The plugin bridge performance baseline also scans `ScriptCallTable::call(...)` and the real backend callback builder so this hot path cannot drift back to `by_name`, `resolve(...)`, or `HostExportRegistry::call_with_capabilities(...)`. That source-structure guard passed independently on 2026-06-14; Cargo lib-test execution is still waiting on unrelated render call-arity compile errors.

## Fixed Modules

| Module | Source | Capabilities | Notes |
|---|---|---|---|
| `zr.zircon.foundation` | `builtin_host_modules.rs` | `foundation.event`, `foundation.log`, `foundation.time` | Process and host utility calls. |
| `zr.zircon.asset` | `builtin_host_modules.rs` | `asset.query` | Locator/status reads only. |
| `zr.zircon.scene` | `builtin_host_modules.rs` | `scene.handle`, `scene.query` | Stable host-handle projection, not raw world access. |
| `zr.zircon.render` | `builtin_host_modules.rs` | `render.query` | Read-only render metadata. |
| `zr.zircon.math` | `builtin_host_modules.rs` macro module | none declared | Pure value descriptors and deterministic math helpers. |
| `zr.zircon.gameplay` | `gameplay_host.rs` | `gameplay.entity`, `gameplay.input`, `gameplay.navigation` | Current gameplay ECS facade through `ScriptRuntimeCallContext`. |

## Foundation

| Function | Parameters | Return | Required capability |
|---|---|---|---|
| `time_unix_millis` | none | `Int` | `foundation.time` |
| `log_info` | `message:String` | `Null` | `foundation.log` |
| `event_publish` | `topic:String`, `payload:String` | `Bool` | `foundation.event` |

## Asset

| Function | Parameters | Return | Required capability |
|---|---|---|---|
| `locator_identity` | `locator:String` | `String` | `asset.query` |
| `status` | `locator:String` | `String` | `asset.query` |
| `revision` | `locator:String` | `Int` | `asset.query` |

## Scene

| Function | Parameters | Return | Required capability |
|---|---|---|---|
| `default_world_handle` | none | `HostHandle` | `scene.handle` |
| `handle_is_valid` | `handle:HostHandle` | `Bool` | `scene.query` |
| `summary` | `handle:HostHandle` | `String` | `scene.query` |

## Render

| Function | Parameters | Return | Required capability |
|---|---|---|---|
| `backend_name` | none | `String` | `render.query` |
| `frame_index` | none | `Int` | `render.query` |

## Math

| Function or type | Parameters or fields | Return/value kind | Required capability |
|---|---|---|---|
| Type `Vec3` | `x:float`, `y:float`, `z:float` | `Float` descriptor, struct prototype | none |
| Type `ColorRgba` | `r:float`, `g:float`, `b:float`, `a:float` | `Float` descriptor, struct prototype | none |
| `vec3_length` | `x:float`, `y:float`, `z:float` | `Float` | none |
| `vec3_dot` | `ax:float`, `ay:float`, `az:float`, `bx:float`, `by:float`, `bz:float` | `Float` | none |

## Gameplay

| Function | Parameters | Return | Required capability |
|---|---|---|---|
| `delta_seconds` | none | `Float` | none |
| `entity` | none | `Int` | `gameplay.entity` |
| `key_pressed` | `key:String` | `Bool` | `gameplay.input` |
| `position_json` | `entity:Int` | `String` | `gameplay.entity` |
| `position_x` | `entity:Int` | `Float` | `gameplay.entity` |
| `position_y` | `entity:Int` | `Float` | `gameplay.entity` |
| `position_z` | `entity:Int` | `Float` | `gameplay.entity` |
| `set_position_json` | `entity:Int`, `position_json:String` | `Bool` | `gameplay.entity` |
| `set_position` | `entity:Int`, `x:Float`, `y:Float`, `z:Float` | `Bool` | `gameplay.entity` |
| `translate_json` | `entity:Int`, `delta_json:String` | `Bool` | `gameplay.entity` |
| `translate` | `entity:Int`, `x:Float`, `y:Float`, `z:Float` | `Bool` | `gameplay.entity` |
| `face_direction` | `entity:Int`, `x:Float`, `z:Float` | `Bool` | `gameplay.entity` |
| `set_scale` | `entity:Int`, `x:Float`, `y:Float`, `z:Float` | `Bool` | `gameplay.entity` |
| `follow_position` | `entity:Int`, `target_entity:Int`, `offset_x:Float`, `offset_y:Float`, `offset_z:Float` | `Bool` | `gameplay.entity` |
| `move_towards_entity` | `entity:Int`, `target_entity:Int`, `speed:Float`, `dt:Float` | `Bool` | `gameplay.entity` |
| `camera_follow` | `entity:Int`, `target_entity:Int`, `offset_x:Float`, `offset_y:Float`, `offset_z:Float` | `Bool` | `gameplay.entity` |
| `component_json` | `entity:Int`, `component_id:String` | `String` | `gameplay.entity` |
| `component_string` | `entity:Int`, `component_id:String`, `fallback:String` | `String` | `gameplay.entity` |
| `set_component_json` | `entity:Int`, `component_id:String`, `component_json:String` | `Bool` | `gameplay.entity` |
| `find_by_component` | `component_id:String` | `String` | `gameplay.entity` |
| `nearest_by_script_property` | `source_entity:Int`, `property:String`, `value:String`, `max_distance:Float` | `Int` | `gameplay.entity` |
| `count_by_script_property` | `property:String`, `value:String` | `Int` | `gameplay.entity` |
| `script_property_matches` | `entity:Int`, `property:String`, `value:String` | `Bool` | `gameplay.entity` |
| `script_number` | `entity:Int`, `property:String`, `fallback:Float` | `Float` | `gameplay.entity` |
| `set_animation_bool` | `entity:Int`, `parameter:String`, `value:Bool` | `Bool` | `gameplay.entity` |
| `damage_entity` | `entity:Int`, `damage:Float` | `Bool` | `gameplay.entity` |
| `heal_entity` | `entity:Int`, `amount:Float`, `max_hp:Float` | `Bool` | `gameplay.entity` |
| `current_hp` | `entity:Int`, `fallback_hp:Float` | `Float` | `gameplay.entity` |
| `damage_entity_report` | `entity:Int`, `damage:Float` | `String` | `gameplay.entity` |
| `spawn_empty` | `name:String`, `position_json:String` | `Int` | `gameplay.entity` |
| `spawn_model` | `name:String`, `position_json:String`, `model_ref:String`, `material_ref:String`, `script_bindings_json:String` | `Int` | `gameplay.entity` |
| `set_hud_text` | `entity:Int`, `text:String` | `Bool` | `gameplay.entity` |
| `set_particle_sprites` | `entity:Int`, `sprites_json:String` | `Bool` | `gameplay.entity` |
| `set_world_hud_bar` | `entity:Int`, `max_hp:Float`, `width:Float`, `height:Float`, `y_offset:Float`, `intensity:Float` | `Bool` | `gameplay.entity` |
| `despawn` | `entity:Int` | `Bool` | `gameplay.entity` |
| `nav_next_point_json` | `start_json:String`, `end_json:String` | `String` | `gameplay.navigation` |
| `nav_move_towards_entity` | `entity:Int`, `target_entity:Int`, `speed:Float`, `dt:Float` | `Bool` | `gameplay.navigation` |

## Dynamic Bridge Module

| Module | Function source | Required capability rule | Runtime 13 judgement |
|---|---|---|---|
| `zr.zircon.bridge` | Caller-supplied `ScriptBridgeMethodDescriptor` rows | Every method starts with `bridge.call`; callers may append method-specific capabilities | Document the module shape in Runtime 13, but do not freeze concrete bridge method names here while the plugin bridge plan owns reflection descriptor generation. |

Each bridge function descriptor is derived from `function_name`, `interface_id`, `method_slot`, parameter descriptors, return kind, and documentation. Calls resolve through `FrozenBridgeTable`, reject absent or disabled interfaces explicitly, and record enabled/not-enabled bridge diagnostics.

## Marshalling Rules

Runtime 13 adopts three script boundary shapes:

| Shape | Allowed values | Current source | Boundary rule |
|---|---|---|---|
| Value descriptors | `Null`, `Bool`, `Int`, `Float`, `String`, `Bytes`, plus macro-derived type descriptors whose fields lower to those kinds | `ScriptHostValue`, `ScriptHostTypeDescriptor`, `ZirconScriptType` | Pass by value only. No raw Rust references, `World`, manager, or backend pointer crosses the VM boundary. |
| Host handles | `HostHandle(u64)` and current gameplay entity IDs represented as `Int` | `handles.rs`, `default_world_handle`, gameplay entity functions | Handles must be validated by the owning runtime surface before use. Runtime 13 M2 will align script-held entity invalidation with Runtime 08 lifecycle tests. |
| Serialized payloads | JSON strings and future `Bytes` payloads for larger opaque data | `position_json`, `component_json`, `script_bindings_json`, nav point JSON | Structured data that is not a small value descriptor crosses as serialized data, not as engine-owned object references. |

`zircon_runtime_interface::reflect` remains the editor/remote reflection schema surface. Its richer `ReflectedValue` and `ReflectSerializationStrategy` taxonomy informs marshalling policy, but VM host calls use `ScriptHostValue` descriptors unless a future bridge explicitly serializes a reflect payload.

## ECS Access Path

The current script gameplay ECS path is `zr.zircon.gameplay` through `ScriptRuntimeCallContext`, which carries `CoreWeak`, `LevelSystem`, the active `EntityId`, and `delta_seconds`. This keeps VM scripts on a gameplay facade rather than exposing raw ECS storage or native plugin ABI tables.

`ZrHostEcsApiV1` belongs to the native/plugin ABI layer. Runtime 13 keeps it separate from the default VM gameplay path. A VM plugin that needs plugin-owned bridge behavior should route through `zr.zircon.bridge` descriptors rather than calling the native ABI table directly.

## Guard Follow-Up

Runtime 13 M1 adds `host_function_registry_matches_documented_ledger` under `runtime_absorption` so the guard can read the host registration sources without editing the plugin bridge lane's active `script/vm/host` files. The guard compares the fixed built-in module descriptors against this ledger and treats `zr.zircon.bridge` as a dynamic module shape contract until plugin-owned method reflection is finalized. `host_function_registry_ledger_guard_rejects_missing_entry` is the negative self-check: removing a fixed function row from this ledger must be detected.

M1.2 extends that guard set with `host_capability_representatives_are_declared_on_registered_modules` and `host_function_without_required_capability_is_rejected_with_explicit_error`. The positive side inspects registered descriptors for one representative from each fixed capability class plus `bridge.call`; the negative side calls those representatives with an empty `CapabilitySet` and requires the explicit `missing capability ...` rejection before callbacks can access gameplay runtime state or bridge providers.

M2 aligns script-held entity ids with the ECS stable-id invalidation rule. `script_held_entity_handle_reports_invalid_after_despawn` exercises the `zr.zircon.gameplay` facade directly: a live id resolves through `position_json`, `despawn` removes it from the `World`, subsequent read access reports `null`, and stale write access is rejected by the world layer instead of recreating the entity.

`script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi` keeps the access-path judgement executable. It scans the current `zircon_runtime/src/script` Rust files for native ECS ABI symbols such as `ZrHostEcsApiV1` and locks the `zr.zircon.gameplay` / `ScriptRuntimeCallContext` anchors as the script-facing gameplay facade.

`script_binding_boundary` mirrors these Runtime 13 facts through the Python structural audit. Current evidence reports audited source files 10/10, guard/test files 2/2, fixed host ledger 6 modules / 50 functions / 2 type descriptors, callback counts builtin=11/11, gameplay=37/37, macro=2/2, host capability anchors 11/11, Runtime 13 guard anchors 7/7, native ECS ABI references in script source = 0, and `risks = []`. This is structure evidence only; `cargo test -p zircon_runtime --lib script --locked -- --nocapture` remains pending.
