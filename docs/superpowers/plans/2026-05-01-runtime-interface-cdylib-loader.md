# Runtime Interface Cdylib Loader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a stable `zircon_runtime_interface` crate first, then make `zircon_runtime` load as a standalone dynamic runtime library, then cut `zircon_editor` and `zircon_plugins` away from source-level runtime/editor implementation dependencies.

**Architecture:** `zircon_runtime_interface` owns the cross-library ABI and DTO surface. `zircon_runtime` exports a versioned function table from a `cdylib`, while `zircon_app` owns process startup, dynamic loading, and host event routing. `zircon_editor` and plugins later depend only on interface crates and communicate through opaque handles, serialized payloads, status values, capability negotiation, and function tables.

**Tech Stack:** Rust 2021, Cargo workspace, `cdylib`, `libloading`, `repr(C)` ABI structs, opaque handles, byte slices, owned byte buffers, existing `zircon_app`, `zircon_runtime`, `zircon_editor`, and independent `zircon_plugins` workspace.

---

## Current Baseline

- Root workspace currently includes `zircon_app`, `zircon_runtime`, and `zircon_editor`.
- `zircon_editor` directly depends on `zircon_runtime` through a path dependency.
- `zircon_app` directly depends on `zircon_runtime` and optionally on `zircon_editor`.
- `zircon_plugins/*/{runtime,editor}` crates directly depend on `zircon_runtime`, `zircon_editor`, or both.
- Existing NativeDynamic plugin ABI proves plugin dynamic loading mechanics, but it is not the runtime host dynamic boundary.
- `zircon_app` runtime preview currently imports runtime implementation types such as `CoreHandle`, `RenderFrameExtract`, `InputManager`, `LevelSystem`, math types, and render framework traits.

## Boundary Rules

- Do not cross dynamic boundaries with `CoreHandle`, `Arc<dyn Trait>`, Rust trait objects, borrowed references, Slint objects, `wgpu::*`, runtime world objects, editor state, or host-owned OS/GPU pointers.
- Only cross boundaries with `repr(C)` structs, primitive values, opaque handle ids, status codes, ABI version numbers, byte slices, owned byte buffers, serialized DTOs, and function tables.
- Runtime keeps `CoreRuntime`, module lifecycle, scene/ECS authority, asset/resource ownership, base render host, and plugin loading internally.
- App keeps process/window/main-loop ownership.
- Editor and plugins use interface clients and do not import runtime/editor implementation crates after their cutover milestones.

## Milestone 0: Interface Contract Foundation

**Goal:** Add the stable ABI crate that runtime, app, editor, and plugins can share without pulling in runtime implementation.

**Files:**

- Create: `zircon_runtime_interface/Cargo.toml`
- Create: `zircon_runtime_interface/src/lib.rs`
- Create: `zircon_runtime_interface/src/version.rs`
- Create: `zircon_runtime_interface/src/status.rs`
- Create: `zircon_runtime_interface/src/handles.rs`
- Create: `zircon_runtime_interface/src/buffer.rs`
- Create: `zircon_runtime_interface/src/runtime_api.rs`
- Create: `zircon_runtime_interface/src/plugin_api.rs`
- Create: `zircon_runtime_interface/src/manifest.rs`
- Create: `zircon_runtime_interface/src/tests/mod.rs`
- Create: `zircon_runtime_interface/src/tests/contracts.rs`
- Modify: `Cargo.toml`
- Create: `docs/engine-architecture/runtime-interface-cdylib-loader.md`

**Implementation slices:**

- [x] Add `zircon_runtime_interface` to root workspace members.
- [x] Define `ZIRCON_RUNTIME_ABI_VERSION_V1` and the exported symbol name constant for the future runtime cdylib.
- [x] Define opaque handles for runtime sessions, viewports, and plugins.
- [x] Define status values and diagnostic byte payload conventions.
- [x] Define byte slice and owned byte buffer ABI types.
- [x] Define the runtime function table shape without implementing runtime behavior.
- [x] Define the plugin ABI entry shape without migrating plugin crates yet.
- [x] Define manifest DTO enums and descriptor records used by later runtime/plugin adapters.
- [x] Add interface crate contract tests.
- [x] Add module documentation under `docs/engine-architecture/`.

**Testing stage:**

- `cargo check -p zircon_runtime_interface --locked`
- `cargo test -p zircon_runtime_interface --locked`
- `cargo tree -p zircon_runtime_interface --locked`

**Exit evidence:**

- Interface crate compiles alone.
- Interface crate has no runtime/editor implementation dependency.
- ABI types are versioned and consumable by app/editor/plugins.

## Milestone 1: Runtime Cdylib Export And App Loader

**Goal:** Build `zircon_runtime` as an independently loadable dynamic library and make `zircon_app` runtime profile load it through `zircon_runtime_interface`.

**Files:**

- Modify: `zircon_runtime/Cargo.toml`
- Create: `zircon_runtime/src/dynamic_api/mod.rs`
- Create: `zircon_runtime/src/dynamic_api/exports.rs`
- Create: `zircon_runtime/src/dynamic_api/session.rs`
- Create: `zircon_runtime/src/dynamic_api/runtime_loop.rs`
- Create: `zircon_runtime/src/dynamic_api/frame.rs`
- Modify: `zircon_runtime/src/lib.rs`
- Modify: `zircon_app/Cargo.toml`
- Create: `zircon_app/src/entry/runtime_library/mod.rs`
- Create: `zircon_app/src/entry/runtime_library/library_path.rs`
- Create: `zircon_app/src/entry/runtime_library/loaded_runtime.rs`
- Create: `zircon_app/src/entry/runtime_library/runtime_session.rs`
- Modify: `zircon_app/src/entry/mod.rs`
- Modify: `zircon_app/src/entry/entry_runner/runtime.rs`
- Modify: `zircon_app/src/entry/runtime_entry_app/**`
- Modify: `zircon_app/src/runtime_presenter.rs`

**Implementation slices:**

- [x] Set `zircon_runtime` lib crate type to include `rlib` and `cdylib`.
- [x] Export `zircon_runtime_get_api_v1` from runtime.
- [x] Add runtime-owned session creation/destruction behind the API table.
- [x] Move runtime preview world, input, camera, and render bridge ownership into runtime session state.
- [x] Keep app-side window/event loop ownership and send only ABI events into runtime.
- [x] Return RGBA frames through interface-owned byte buffers.
- [x] Resolve runtime library path from `ZIRCON_RUNTIME_LIBRARY`, then executable sibling names.

**Testing stage:**

- `cargo build -p zircon_runtime --lib --locked`
- `cargo check -p zircon_app --features target-client --locked`
- `cargo test -p zircon_runtime --lib dynamic_api --locked`
- `cargo test -p zircon_app --lib runtime_library --locked`

**Exit evidence:**

- Runtime cdylib is produced.
- `zircon_app` runtime profile loads runtime through the interface table.
- Runtime profile no longer imports runtime implementation objects directly from app code.

## Milestone 2: Editor Runtime Client Cutover

**Goal:** Remove `zircon_editor` source dependency on `zircon_runtime`; editor uses `zircon_runtime_interface` for runtime session access.

**Implementation slices:**

- [ ] Replace editor `zircon_runtime` dependency with `zircon_runtime_interface`.
- [ ] Add `EditorRuntimeClient` as an editor-owned wrapper over runtime interface handles.
- [ ] Replace direct runtime world/default-level/manager access with serialized commands and capability queries.
- [ ] Load runtime cdylib before editor startup and pass the editor runtime client into editor host code.

**Testing stage:**

- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo tree -p zircon_editor --locked`

**Exit evidence:**

- `zircon_editor` compiles without compiling `zircon_runtime`.
- Editor can start with runtime loaded dynamically.

## Milestone 3: App Static Runtime Dependency Removal

**Goal:** Make `zircon_app` depend on runtime implementation only through dynamic loading for runtime/editor host paths.

**Implementation slices:**

- [ ] Move app-facing runtime config/profile DTOs into interface crate where needed.
- [ ] Delete app code that constructs `CoreRuntime` or `BuiltinEngineEntry` directly.
- [ ] Replace static bootstrap methods with dynamic runtime session creation.
- [ ] Remove `zircon_runtime` from `zircon_app/Cargo.toml` after no static imports remain.

**Testing stage:**

- `cargo check -p zircon_app --features target-client --locked`
- `cargo check -p zircon_app --features target-editor-host --locked`
- `cargo test -p zircon_app --locked`
- `cargo tree -p zircon_app --features target-client --locked`
- `cargo tree -p zircon_app --features target-editor-host --locked`

## Milestone 4: Plugin Interface Substrate

**Goal:** Define the plugin-side runtime ABI that lets plugin crates compile against interface only.

**Implementation slices:**

- [ ] Extend plugin ABI descriptor records for package manifest, target modes, capabilities, service descriptors, command descriptors, event descriptors, and state callbacks.
- [ ] Add runtime-side adapter from interface plugin descriptors to internal runtime registries.
- [ ] Keep the existing NativeDynamic behavior table aligned with these interface types.

**Testing stage:**

- `cargo test -p zircon_runtime_interface --locked`
- `cargo test -p zircon_runtime --lib native_plugin_loader --locked`
- `cargo test -p zircon_runtime --test native_plugin_loader_contract --locked`
- `cargo test -p zircon_runtime --test export_build_plan_contract --locked`

## Milestone 5: Low-Risk Plugin Cutover Pilot

**Goal:** Prove one plugin can compile without `zircon_runtime` and load dynamically through the new interface.

**Recommended pilot:** `zircon_plugins/native_dynamic_fixture/native`, then one simple runtime plugin such as `texture`, `navigation`, or `particles`.

**Testing stage:**

- `cargo check --manifest-path zircon_plugins/Cargo.toml -p <pilot_plugin> --locked`
- `cargo tree --manifest-path zircon_plugins/Cargo.toml -p <pilot_plugin> --locked`
- `cargo test -p zircon_runtime --lib plugin_extensions --locked`
- `cargo test -p zircon_runtime --test native_plugin_loader_contract --locked`

## Milestone 6: Editor Plugin Interface Cutover

**Goal:** Make editor plugin crates stop depending on `zircon_editor` implementation.

**Implementation slices:**

- [ ] Add `zircon_editor_interface` only if editor ABI cannot live cleanly in `zircon_runtime_interface`.
- [ ] Define editor view/menu/operation/template/drawer contribution ABI records.
- [ ] Keep Slint objects, editor state, workbench internals, and Rust operation enums inside `zircon_editor`.
- [ ] Convert one editor plugin and then the full editor plugin set.

**Testing stage:**

- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support --locked`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_runtime_diagnostics_editor --locked`
- `cargo tree --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_runtime_diagnostics_editor --locked`
- `cargo test -p zircon_editor --lib editor_plugin --locked`

## Milestone 7: Full Plugin Workspace Cutover

**Goal:** Remove source-level runtime/editor implementation dependencies from all `zircon_plugins`.

**Implementation slices:**

- [ ] Convert simple plugins first: `texture`, `navigation`, `particles`, `sound`, `net`.
- [ ] Keep `physics` and `animation` out of this plugin cutover; those domains are runtime-owned under `zircon_runtime::{physics,animation}`.
- [ ] Convert render-heavy plugins last: `virtual_geometry`, `hybrid_gi`.
- [ ] Add structural dependency guards.

**Testing stage:**

- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- `cargo test -p zircon_runtime --lib plugin_extensions --locked`
- `cargo test -p zircon_editor --lib editor_plugin --locked`

## Milestone 8: Final Validation And Hard-Cut Cleanup

**Goal:** Prove the new architecture is the only live path and remove migration residue.

**Implementation slices:**

- [ ] Remove migration-only shims, compatibility modules, bridge modules, and old static plugin registration paths.
- [ ] Update docs and structural dependency guards.
- [ ] Keep root wiring files structural.

**Testing stage:**

- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1`
