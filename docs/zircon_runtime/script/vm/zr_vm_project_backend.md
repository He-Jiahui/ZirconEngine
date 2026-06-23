---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/compiler.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/module.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/library/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/compiler.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/module.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/library/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
tests:
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - cargo test -p zircon_runtime --lib fallback_project_backend_does_not_claim_real_zr_vm_selector --message-format short --color never
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs::bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_w_key_moves_player_before_input_clear --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_reports_runtime_fps_and_render_work --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, ZR_VAMPIRE_CAPTURE_PNG, ZR_VAMPIRE_CAPTURE_WIDTH=640, and ZR_VAMPIRE_CAPTURE_HEIGHT=360 set
  - cargo build -q -p zircon_app --bin zircon_runtime --features first-party-zr-vm-real-backend
  - target\debug\zircon_runtime.exe --log-level verbose --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR, PATH, and ZIRCON_RUNTIME_LIBRARY set for the local debug ZR VM build
  - cargo test --manifest-path ..\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml call_module_export_accepts_empty_argument_slice -- --nocapture
  - cargo test --manifest-path ..\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml project_session_preserves_module_state_between_export_calls -- --nocapture
  - vm_lifecycle_fallback_activate_bad_entry_module_surfaces_vm_error
  - vm_lifecycle_fallback_missing_optional_export_returns_none_not_error
  - vm_lifecycle_fallback_deactivate_is_idempotent_after_unload
  - vm_lifecycle_fallback_empty_arguments_do_not_require_real_backend
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata
  - zircon_runtime/src/asset/tests/project/zmeta.rs::project_manager_imports_compound_zshader_package_with_subassets
doc_type: module-detail
---

# ZrVM Project Backend

## Purpose

`zircon_runtime::script::vm::backend::zr_vm_project_backend` is the runtime-owned real backend for project script packages that declare `backend = "zr_vm:project"`. It lets the standalone runtime load project-local ZR script packages without depending on the external plugin runtime crate as the owner of the VM boundary.

The real backend is feature gated through `zircon_runtime/zr-vm-real-backend`. `zircon_app` exposes this through `first-party-zr-vm-real-backend`, and `zircon_plugins/zr_vm_language/runtime` now forwards to the runtime backend instead of owning a separate implementation.

## Backend Selection

`zr_vm:project` is now claimed by `ZrVmBackend`. The fallback project backend is intentionally narrowed to `zr_vm_fallback:project` and `fallback:project`, so a package that opts into the real backend no longer silently runs through fallback gameplay code.

The selected backend compiles or reuses cached project modules, creates a runtime plugin instance, and provides the same lifecycle surface expected by `VmPluginManager`: `activate`, `deactivate`, `save_state`, `restore_state`, and exported scene callbacks such as `onStart` and `onUpdate`.

## Host Callback Call Table

Before registering host modules into `zr_vm`, the real backend now snapshots `HostExportRegistry` into a dense `ScriptCallTable`. Each generated native host function captures a pre-resolved `ScriptCallSite`, so the callback path keeps arity/capability validation but skips repeated module/function name lookup. This also covers the dynamic `zr.zircon.bridge` module because bridge methods are registered through the same host export ledger before backend module registration. `bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites` keeps that as an executable structure guard by requiring registration-time resolution before `build_native_function(...)` and forbidding callback-time `resolve(...)` or `call_with_capabilities(...)`.

## Lifecycle Export Boundary

`ZrVmPluginInstance` calls entry lifecycle exports through the ZrVM session path. `activate`, `deactivate`, and `saveState` intentionally pass zero script arguments; `restoreState` passes one serialized state string. The zero-argument calls must remain count `0`, not a synthetic `null` value, because the script function signatures are no-argument lifecycle hooks.

The lower shared boundary is the local `../../zr_vm` Rust binding. Its module export marshalling now passes a valid sentinel argument-pointer base while keeping `argumentCount == 0` for empty slices. This prevents the native VM from asserting on a null argument pointer before it observes the zero count, and it fixes both `ProjectWorkspace::call_module_export` and `ProjectSession::call_module_export`. The binding regression anchors are `call_module_export_accepts_empty_argument_slice` and the existing session lifecycle test `project_session_preserves_module_state_between_export_calls`.

The runtime wrapper must keep using the normal `call_entry_lifecycle_export(..., &[])` path. Adding a dummy `Value::new_null()` at the runtime layer would change the ABI-visible lifecycle arity and hide the shared FFI bug instead of fixing it.

## Vampire Script Path

The current vampire example is the first real project package that exercises this path with gameplay callbacks. Its `plugin.toml` uses `backend = "zr_vm:project"`, and `main.zr` imports the reflected gameplay host module before driving `onStart` and `onUpdate` through generic calls such as `key_pressed`, `translate`, `face_direction`, `set_world_hud_bar`, `camera_follow`, `follow_position`, `damage_entity`, `set_particle_sprites`, `entity_exists`, `script_number_at_most`, and `nav_move_towards_entity`.

Local run validation exposed two real-VM script constraints while bringing up the vampire package. First, direct host calls from exported callbacks execute, but routing the same calls through local script helper functions can still trigger a native `zr_vm_core.dll` access violation. Second, some runtime multiplication expressions are rejected as `MUL_FLOAT requires numeric operands`; the current script avoids those expressions and leaves time-scaled movement/navigation work to host calls where possible.

The real backend currently also avoids direct ZR script comparisons against host-returned entity ids or numeric component values in the vampire gameplay path. `gameplay.entity_exists(...)` and `gameplay.script_number_at_most(...)` keep those checks inside the Rust gameplay host, which is the validated side of the boundary. The Retry path additionally uses a one-frame `vampire.spawn_grace` component so enemies cannot apply another lethal hit in the same frame that the player is reset.

This means the runtime can now host and render a simple 3D game package through the real project backend, but script authoring should stay inside the validated subset until script-to-script helper calls and numeric expression typing are fixed in the VM boundary. The vampire scene keeps the real VM hot path to the player plus three active enemy bindings; additional visible enemies are authored with disabled bindings so they remain rendered content without becoming callback targets.

## Validation Notes

On 2026-06-13, the VM host callback table slice passed two focused runtime tests against `D:\cargo-targets\zircon-plugin-architecture-bridge-0613`: `script_call_table_pre_resolves_host_export_callbacks` and `zr_vm_real_backend_uses_script_call_table_for_host_callbacks`. The same slice also passed `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never` with existing warning noise.

On 2026-06-14, the plugin bridge performance baseline added `bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites` as a source-structure guard for the same real-backend callback path. The independent source-structure check passed, proving the current source still resolves call sites before callback construction and keeps callback dispatch on `ScriptCallSite::call(...)`. It is a performance-regression guard, not a replacement for the runtime behavior tests above; Cargo lib-test validation is currently blocked before target execution by unrelated render call-arity errors.

On 2026-06-12, the lower binding regression tests passed against `E:\Git\zr_vm\build-msvc\bin\Release` / `lib\Release`: `call_module_export_accepts_empty_argument_slice` and `project_session_preserves_module_state_between_export_calls` both completed with one passing test. This proves the binding no longer turns empty export argument slices into a native null argument pointer for workspace or session export calls.

On 2026-06-16, Runtime 06 M1.2 added folder-backed fallback lifecycle failure tests under `zircon_runtime/src/script/vm/tests/lifecycle_failures.rs`, mounted from `zircon_runtime/src/script/vm/tests.rs` so the already-large VM test owner does not absorb another responsibility. The four tests cover bad entry-module activation errors, missing optional export returning `None`, idempotent deactivate after unload, and empty argument calls without the real backend. This is fallback lifecycle failure tests 4/4 for the static/fallback layer. The focused command `cargo test -p zircon_runtime --lib vm_lifecycle_fallback --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-06-fallback-lifecycle-0616 -- --nocapture --test-threads=1` passed 5/5 after the render pipeline compile test module imported the public `PostProcessStackDescriptor` and `RenderPipelineCompileOptions` types it already used; real-backend Cargo remains pending.

The 2026-06-16 real-backend retry used the Debug ZR VM build under `E:\Git\zr_vm\build-msvc` and moved past the previous missing `ZR_VM_RUST_BINDING_LIB_DIR` setup. It then exposed a shader artifact cache boundary in the vampire project: `lib://shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset` failed with `tag for enum is not valid, found 5`. The fix lives in `zircon_runtime/src/asset/artifact/cache_payload.rs`, where shader import redirects and texture slots now use cache-local bincode-safe structs, and `examples/vampire/assets/shaders/default_pbr.zmeta` plus its `.zasset` were regenerated. Focused real-backend validation passed `artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata` 1/1 and `project_manager_imports_compound_zshader_package_with_subassets` 1/1.

After that cache fix, the broader `cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1` no longer fails at artifact parse but timed out after 604 seconds during the first retry window. That result is not a runtime assertion failure and not a pass for the full group.

The current focused Release-ZrVM validation passed the two menu lifecycle paths that were blocking the sample: `vampire_project_session_starts_paused_until_start_button_click` passed after the test helper sends the matching viewport resize before the synthetic click, and `vampire_project_session_game_over_menu_retries_to_playing` passed after the gameplay host gained `entity_exists` / `script_number_at_most` and the script gained the one-frame `vampire.spawn_grace` retry guard. The full `vampire_project_session` group, plugin/native plugin, app, and `zircon_plugins` workspace gates remain pending.

The latest real VM offscreen validation uses the Release ZR VM DLLs from `E:\Git\zr_vm\build-msvc\bin\Release` and exports `examples/vampire/screenshots/vampire-runtime-playable-current-640.png`. The process loads the project, reuses or compiles `vampire_game`, executes one update tick through the real backend, and renders a visible scene with scene-following HUD bars. The matching diagnostic test reported `fps_current=60.872053031732605`, `frame_ms_current=16.427899999999998`, one submitted frame, 116 mesh draws, and zero screen-space UI commands.

A 1280x720 screenshot export with one script tick currently exits with Windows status `-1073741819` before the PNG write. The accepted screenshot path is therefore the 640x360 capture until the high-resolution capture boundary is debugged separately.
