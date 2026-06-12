---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/compiler.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/module.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/mod.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/compiler.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/module.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_fallback_backend.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
tests:
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - cargo test -p zircon_runtime --lib fallback_project_backend_does_not_claim_real_zr_vm_selector --message-format short --color never
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_w_key_moves_player_before_input_clear --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_reports_runtime_fps_and_render_work --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR set
  - target\debug\deps\zircon_runtime-c2d0caf045e075d5.exe vampire_project_session_capture_frame_draws_world_hud_bars --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR, ZR_VAMPIRE_CAPTURE_PNG, ZR_VAMPIRE_CAPTURE_WIDTH=640, and ZR_VAMPIRE_CAPTURE_HEIGHT=360 set
  - cargo build -q -p zircon_app --bin zircon_runtime --features first-party-zr-vm-real-backend
  - target\debug\zircon_runtime.exe --log-level verbose --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR, PATH, and ZIRCON_RUNTIME_LIBRARY set for the local debug ZR VM build
  - cargo test --manifest-path ..\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml call_module_export_accepts_empty_argument_slice -- --nocapture
  - cargo test --manifest-path ..\zr_vm\zr_vm_rust_binding\rust\zr_vm_rust_binding\Cargo.toml project_session_preserves_module_state_between_export_calls -- --nocapture
doc_type: module-detail
---

# ZrVM Project Backend

## Purpose

`zircon_runtime::script::vm::backend::zr_vm_project_backend` is the runtime-owned real backend for project script packages that declare `backend = "zr_vm:project"`. It lets the standalone runtime load project-local ZR script packages without depending on the external plugin runtime crate as the owner of the VM boundary.

The real backend is feature gated through `zircon_runtime/zr-vm-real-backend`. `zircon_app` exposes this through `first-party-zr-vm-real-backend`, and `zircon_plugins/zr_vm_language/runtime` now forwards to the runtime backend instead of owning a separate implementation.

## Backend Selection

`zr_vm:project` is now claimed by `ZrVmBackend`. The fallback project backend is intentionally narrowed to `zr_vm_fallback:project` and `fallback:project`, so a package that opts into the real backend no longer silently runs through fallback gameplay code.

The selected backend compiles or reuses cached project modules, creates a runtime plugin instance, and provides the same lifecycle surface expected by `VmPluginManager`: `activate`, `deactivate`, `save_state`, `restore_state`, and exported scene callbacks such as `onStart` and `onUpdate`.

## Lifecycle Export Boundary

`ZrVmPluginInstance` calls entry lifecycle exports through the ZrVM session path. `activate`, `deactivate`, and `saveState` intentionally pass zero script arguments; `restoreState` passes one serialized state string. The zero-argument calls must remain count `0`, not a synthetic `null` value, because the script function signatures are no-argument lifecycle hooks.

The lower shared boundary is the local `../../zr_vm` Rust binding. Its module export marshalling now passes a valid sentinel argument-pointer base while keeping `argumentCount == 0` for empty slices. This prevents the native VM from asserting on a null argument pointer before it observes the zero count, and it fixes both `ProjectWorkspace::call_module_export` and `ProjectSession::call_module_export`. The binding regression anchors are `call_module_export_accepts_empty_argument_slice` and the existing session lifecycle test `project_session_preserves_module_state_between_export_calls`.

The runtime wrapper must keep using the normal `call_entry_lifecycle_export(..., &[])` path. Adding a dummy `Value::new_null()` at the runtime layer would change the ABI-visible lifecycle arity and hide the shared FFI bug instead of fixing it.

## Vampire Script Path

The current vampire example is the first real project package that exercises this path with gameplay callbacks. Its `plugin.toml` uses `backend = "zr_vm:project"`, and `main.zr` imports the reflected gameplay host module before driving `onStart` and `onUpdate` through generic calls such as `key_pressed`, `translate`, `face_direction`, `set_world_hud_bar`, `camera_follow`, `follow_position`, `damage_entity`, `set_particle_sprites`, and `nav_move_towards_entity`.

Local run validation exposed two real-VM script constraints while bringing up the vampire package. First, direct host calls from exported callbacks execute, but routing the same calls through local script helper functions can still trigger a native `zr_vm_core.dll` access violation. Second, some runtime multiplication expressions are rejected as `MUL_FLOAT requires numeric operands`; the current script avoids those expressions and leaves time-scaled movement/navigation work to host calls where possible.

This means the runtime can now host and render a simple 3D game package through the real project backend, but script authoring should stay inside the validated subset until script-to-script helper calls and numeric expression typing are fixed in the VM boundary. The vampire scene keeps the real VM hot path to the player plus three active enemy bindings; additional visible enemies are authored with disabled bindings so they remain rendered content without becoming callback targets.

## Validation Notes

On 2026-06-12, the lower binding regression tests passed against `E:\Git\zr_vm\build-msvc\bin\Release` / `lib\Release`: `call_module_export_accepts_empty_argument_slice` and `project_session_preserves_module_state_between_export_calls` both completed with one passing test. This proves the binding no longer turns empty export argument slices into a native null argument pointer for workspace or session export calls.

The runtime-level real-backend command `cargo test -p zircon_runtime --lib vampire_project_session_starts_paused_until_start_button_click --features zr-vm-real-backend --locked -- --nocapture --test-threads=1` timed out after 300 seconds while compiling in an active shared workspace. That result is not a runtime assertion failure and not a pass; it remains the next required validation for the full `ZrVmPluginInstance` path.

The latest real VM offscreen validation uses the Release ZR VM DLLs from `E:\Git\zr_vm\build-msvc\bin\Release` and exports `examples/vampire/screenshots/vampire-runtime-playable-current-640.png`. The process loads the project, reuses or compiles `vampire_game`, executes one update tick through the real backend, and renders a visible scene with scene-following HUD bars. The matching diagnostic test reported `fps_current=60.872053031732605`, `frame_ms_current=16.427899999999998`, one submitted frame, 116 mesh draws, and zero screen-space UI commands.

A 1280x720 screenshot export with one script tick currently exits with Windows status `-1073741819` before the PNG write. The accepted screenshot path is therefore the 640x360 capture until the high-resolution capture boundary is debugged separately.
