---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_library/mod.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_editor/src
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_library/mod.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src
plan_sources:
  - user: 2026-05-01 request runtime/editor/plugin compile isolation through interface crate plus runtime cdylib
  - docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md
  - docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - cargo check -p zircon_runtime_interface --locked
  - cargo test -p zircon_runtime_interface --locked
  - cargo tree -p zircon_runtime_interface --locked
  - cargo build -p zircon_runtime --lib --locked
  - cargo check -p zircon_app --features target-client --locked
  - cargo test -p zircon_runtime --lib dynamic_api --locked
  - cargo test -p zircon_app --lib runtime_library --locked
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-m2-editor --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-m2-editor --message-format short --color never
doc_type: module-detail
---

# Runtime Interface Cdylib Loader

## Purpose

`zircon_runtime_interface` is the stable contract crate for decoupling runtime implementation builds from app, editor, and plugin consumers. It exists so `zircon_runtime` can become a standalone dynamic runtime library while upper layers compile against a small ABI and DTO surface instead of the full runtime implementation crate.

## Ownership

- `zircon_runtime_interface` owns cross-library structs, handles, status values, buffer ownership rules, runtime API tables, plugin API tables, and manifest DTOs.
- `zircon_runtime` will own concrete `CoreRuntime`, scene/ECS authority, asset/resource managers, render host state, plugin loading, and all conversion from interface DTOs into internal descriptors.
- `zircon_app` will own process startup, executable profile selection, OS windows, and dynamic library loading.
- `zircon_editor` and `zircon_plugins` will eventually depend on this interface instead of importing `zircon_runtime` implementation modules.

## ABI Rules

The interface is deliberately narrower than the existing Rust module contracts. Dynamic boundaries must not pass Rust trait objects, `Arc`, borrowed references, Slint objects, `wgpu` objects, runtime world data, editor state, or raw host-owned OS/GPU resources. They pass only `repr(C)` values, primitive ids, handles, status codes, byte slices, owned byte buffers, serialized payloads, and versioned function tables.

## Interface Surface

- `version.rs` defines `ZIRCON_RUNTIME_ABI_VERSION_V1`.
- `handles.rs` defines zero-invalid opaque runtime, viewport, and plugin handles.
- `status.rs` defines raw status codes and diagnostic byte payload attachment.
- `buffer.rs` defines borrowed byte slices and plugin/runtime-owned byte buffers with explicit free callbacks.
- `runtime_api.rs` defines the runtime dynamic library symbol, the v1 runtime function table shape, fixed event records, viewport sizing records, frame requests, and typed captured-frame results.
- `plugin_api.rs` defines the future plugin entry symbol and v1 plugin entry report shape.
- `manifest.rs` defines target mode, module kind, and module descriptor DTO seeds for later runtime/plugin adapters.
- `ui/mod.rs` exposes the shared neutral Runtime UI contract namespace for editor-facing UI DTOs: `binding`, `component`, `dispatch`, `event_ui`, `layout`, `surface`, `template`, and `tree`. The UI namespace is now backed by real `zircon_runtime_interface/src/ui/**` files instead of path-including `zircon_runtime/src/ui/**`. Runtime-only behavior such as component registries, event managers, dispatchers, layout passes, render extraction, text layout, tree mutation, template loading, compiling, and validation remains owned by `zircon_runtime`.

## Milestone 1 Runtime Cdylib

`zircon_runtime` now declares `crate-type = ["rlib", "cdylib"]` and exposes `zircon_runtime_get_api_v1` from `zircon_runtime::dynamic_api`. The exported symbol returns a versioned `ZrRuntimeApiV1` table after checking the host ABI version.

The dynamic runtime session owns the concrete runtime implementation objects that previously lived in `zircon_app` runtime preview code:

- `CoreRuntime` and activated target-client runtime modules.
- Default level/world state and selected-node orbit target.
- Runtime camera interaction state.
- Input manager event routing.
- Render framework viewport creation, extract submission, and captured-frame retrieval.

The ABI boundary receives only `ZrRuntimeEventV1` values for viewport resize, pointer motion, mouse button, and mouse wheel input. It returns `ZrRuntimeFrameV1`, whose `rgba` field is a runtime-owned byte buffer with an explicit free callback.

## Milestone 1 App Loader

`zircon_app` runtime profile now loads runtime with `libloading` instead of bootstrapping runtime preview internals directly. The loader resolves the library path from `ZIRCON_RUNTIME_LIBRARY` first, then falls back to an executable-sibling platform name: `zircon_runtime.dll`, `libzircon_runtime.dylib`, or `libzircon_runtime.so`. Development builds launched directly from Cargo target directories also check executable-sibling `deps/<platform runtime library>` when the packaged sibling library has not been staged yet.

`RuntimeEntryApp` now owns only the window, softbuffer presenter, dynamic runtime session wrapper, viewport handle, and current viewport size. Winit events are converted to interface events and sent to runtime. Redraw requests call the runtime function table `capture_frame` and blit the returned RGBA bytes through softbuffer.

## Validation

Milestone 0 validation is intentionally scoped to the interface crate. The required checks prove the crate compiles by itself, its contract tests pass, and it does not pull in `zircon_runtime`, `zircon_editor`, `wgpu`, `slint`, or plugin implementation crates through dependencies.

Milestone 1 validation adds runtime library build coverage, app target-client checking, and scoped dynamic API/runtime-loader tests. The validation must prove the cdylib export is available, the app runtime profile uses the interface table, and app runtime preview source no longer imports runtime implementation preview objects.

Milestone 2 first-slice validation is scoped to the shared UI contract namespace and editor library type checking. The interface crate check proves the real interface-owned UI contract modules compile without depending on `zircon_runtime`, `zircon_editor`, Slint, wgpu, or plugin crates. The editor library check proves the current editor UI host can type-check after the interface tree split, but it does not prove the editor import cutover is complete: a 2026-05-02 audit found 134 `zircon_runtime::ui` hits and 431 `zircon_runtime_interface::ui` hits in `zircon_editor/src`. The residual runtime hits must be split by role: neutral DTOs should move to `zircon_runtime_interface::ui`, while concrete services such as `UiSurface`, `UiEventManager`, `UiDocumentCompiler`, `UiAssetLoader`, `UiTemplateSurfaceBuilder`, `UiTemplateBuildError`, `UiComponentDescriptorRegistry`, `UiAssetDocumentRuntimeExt`, and `UiPointerDispatcher` remain runtime behavior dependencies. An earlier `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never` passed with existing warnings, and the 2026-05-02 19:44 current-worktree rerun `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never` also passed with existing runtime graphics warnings and 3 editor warnings. `cargo tree -p zircon_editor --locked --depth 1` still lists direct `zircon_runtime` and `zircon_runtime_interface` dependencies for the documented service/contract split.

The `UiSurface` and `UiTree` storage identity has now converged. `zircon_runtime_interface::ui::tree` owns serializable tree contract DTOs, and `zircon_runtime::ui::surface::UiSurface` stores the interface `UiTree` directly. Runtime still owns insertion, mutation, focus, hit-test, scroll, render-order, and routing behavior through `zircon_runtime::ui::tree::UiRuntimeTree*Ext` traits and helper services, so editor surface builders import tree DTOs from the interface crate and import runtime extension traits only when they call behavior methods.
