---
related_code:
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
plan_sources:
  - user: 2026-05-24 continue ZrVM lifecycle diagnostics after real-backend hardening
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/superpowers/specs/2026-05-21-zrvm-real-backend-hardening-design.md
  - docs/superpowers/plans/2026-05-21-zrvm-real-backend-hardening.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-lifecycle-diagnostics
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-lifecycle-diagnostics lifecycle -- --nocapture --test-threads=1
doc_type: design-spec
---

# ZrVM Lifecycle Diagnostics Design

## Purpose

The real ZrVM backend now validates descriptor lowering and callback conversion with contextual errors. The next narrow hardening slice is lifecycle diagnostics: when `activate`, `deactivate`, `saveState`, or `restoreState` runs through the real backend, errors should identify the entry module and lifecycle export that failed while preserving optional-export behavior.

This slice stays inside `zircon_plugins/zr_vm_language/runtime`. It does not change `HotReloadCoordinator`, package discovery, shared descriptors, host modules, capability enforcement, or public backend APIs.

## Current Baseline

`ZrVmPluginInstance` calls optional exports through `call_optional_export(export_name, arguments)`. Missing exports return `Ok(None)` and are intentionally tolerated. Non-missing binding errors are mapped through `map_zr_error`, which reports `zr_vm binding error: ...` but does not say which entry module or lifecycle export failed.

`save_state` already maps missing `saveState` to an empty `VmStateBlob`, string returns to UTF-8 bytes, null returns to empty state, and unsupported value kinds to `zr_vm saveState returned unsupported value kind ...`. `restore_state` already rejects non-UTF-8 state before constructing a ZrVM string argument.

## Chosen Design

Add a private lifecycle label helper in `real_backend.rs`:

```rust
fn lifecycle_export_label(entry_module: &str, export_name: &str) -> String {
    format!("{entry_module}.{export_name}")
}
```

Update `call_optional_export` so binding errors are mapped with that label:

```rust
Err(error) => Err(map_lifecycle_export_error(&self.entry_module, export_name, error)),
```

The new private mapper should return `VmError::Operation` with this shape:

```text
zr_vm lifecycle export main.activate failed: zr_vm binding error: ...
```

Missing optional exports remain non-errors. The missing-export predicate should stay private and should continue to treat `ZR_RUST_BINDING_STATUS_NOT_FOUND`, `not found`, and `NOT_FOUND` as optional absence because the binding can report missing symbols through either status or message text.

Wrap `saveState` unsupported return errors with the entry module label:

```text
zr_vm lifecycle export main.saveState returned unsupported value kind Int
```

Keep `restoreState` UTF-8 validation as host-side validation before a binding call. It should continue to mention `restoreState` and the UTF-8 problem; adding `entry_module.restoreState` context is allowed if implementation can do so without moving validation out of `restore_state`.

## Architecture Boundary

This is a leaf detail inside the existing VM plugin path. The owning abstraction is `VmPluginInstance`; `ZrVmPluginInstance` remains the concrete real-backend adapter. Shared runtime lifecycle orchestration stays in `HotReloadCoordinator`, and shared VM package loading stays in `VmPluginManager`.

No new descriptor, host capability, manager, public trait method, package manifest field, or JSON export is introduced. The change only improves backend-local diagnostics for the concrete ZrVM binding boundary.

## Reference Evidence

Godot `dev/godot/core/extension/gdextension_manager.cpp` keeps extension lifecycle errors at the extension-manager boundary: load, reload, unload, already-loaded, not-loaded, and needs-restart statuses are returned by the lifecycle manager instead of leaking as opaque lower-level failures. This supports adding lifecycle-export context where ZirconEngine crosses into the ZrVM binding.

Bevy `dev/bevy/crates/bevy_app/src/app.rs` and `dev/bevy/crates/bevy_app/src/sub_app.rs` keep plugin lifecycle stages (`build`, readiness, `finish`, `cleanup`) explicit and identify plugin names in tracing spans or duplicate-plugin errors. This supports preserving stage names and plugin/export context in ZrVM lifecycle diagnostics.

The ZrVM binding `E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs` exposes `ProjectSession::call_module_export(module_name, export_name, arguments)`, so the real backend has the exact module/export context at the boundary and can report it without changing lower binding APIs.

## Testing Plan

Add focused tests under the existing `#[cfg(test)]` module in `real_backend.rs` where private helpers are visible and the file only compiles with `real-zr-vm`.

Required coverage:

- lifecycle labels format as `entry_module.export`;
- non-missing binding errors mapped by the lifecycle mapper include `zr_vm lifecycle export`, the label, and the original binding message;
- missing optional exports remain classified as optional absence for `NOT_FOUND` status and known message forms;
- unsupported `saveState` return kind error includes `entry_module.saveState` context;
- `restore_state` non-UTF-8 error continues to reject before calling the binding and includes restore lifecycle context.

Existing real-backend integration tests should continue to cover successful `activate`, `deactivate`, `saveState`, `restoreState`, hot reload, and the checked-in minimal example.

## Validation Plan

Use the same scoped target-dir discipline as the previous ZrVM slice, but with a separate target dir:

```powershell
rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-lifecycle-diagnostics
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-lifecycle-diagnostics lifecycle -- --nocapture --test-threads=1
```

The real-backend command requires `ZR_VM_RUST_BINDING_LIB_DIR` and a matching `PATH` entry for the ZrVM DLL on Windows. If those local artifacts are unavailable, record the exact blocker and do not claim real-backend validation passed.

## Documentation Plan

Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` after implementation to describe lifecycle export diagnostics and record exact validation evidence. Keep this as the existing owner document because it already documents ZrVM package protocol, real backend lowering, and lifecycle conventions.

## Acceptance Criteria

- Binding errors from lifecycle export calls include `entry_module.export` context.
- Missing optional lifecycle exports remain accepted and do not become hard failures.
- `saveState` unsupported return kind diagnostics include lifecycle export context.
- `restoreState` invalid UTF-8 diagnostics include restore lifecycle context.
- Public backend APIs, package protocol, shared descriptor DTOs, host modules, and `HotReloadCoordinator` behavior remain unchanged.
- Scoped non-real plugin tests pass, and real-backend lifecycle tests pass when local ZrVM binding artifacts are configured.
