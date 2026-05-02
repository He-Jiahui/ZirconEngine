---
related_code:
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/asset_uuid.rs
  - zircon_runtime_interface/src/resource/diagnostic.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
  - zircon_runtime_interface/src/resource/resource_event_kind.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/stable_uuid.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime_interface/src/resource/untyped_handle.rs
  - zircon_runtime/src/core/resource/mod.rs
implementation_files:
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/asset_uuid.rs
  - zircon_runtime_interface/src/resource/diagnostic.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
  - zircon_runtime_interface/src/resource/resource_event_kind.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/stable_uuid.rs
  - zircon_runtime_interface/src/resource/state.rs
  - zircon_runtime_interface/src/resource/untyped_handle.rs
plan_sources:
  - user: 2026-05-02 keep zircon_runtime_interface as strict ABI/DTO/serialization contract layer
tests:
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - cargo test -p zircon_runtime --lib core::resource --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline
doc_type: module-detail
---

# Runtime Interface Resource Contracts

## Purpose

`zircon_runtime_interface::resource` owns resource identity and status DTOs that must be shared by the app host, runtime, editor, serialized project files, and future plugin SDK surfaces without pulling in runtime implementation code.

This module is deliberately limited to contract data: locators, stable IDs, typed and untyped handles, resource kind markers, records, state, diagnostics, and resource event rows. Runtime behavior such as IO, caches, leases, registries, managers, project synchronization, importer execution, and payload lifetime control remains in `zircon_runtime::core::resource` and adjacent runtime asset modules.

## Behavior Model

The interface resource surface exposes:

- `ResourceLocator` for normalized `res://`, `lib://`, `builtin://`, and `mem://` locator strings.
- `AssetUuid` and `ResourceId` for stable identity derivation where serialized resources need deterministic IDs.
- `ResourceKind`, marker structs, `ResourceHandle<T>`, and `UntypedResourceHandle` for typed resource references without runtime object ownership.
- `ResourceRecord`, `ResourceState`, `ResourceDiagnostic`, and `ResourceEvent` for serialized status and synchronization reports. `ResourceRecord` carries importer-facing status fields (`source_hash`, `importer_id`, `importer_version`, and `config_hash`) because asset pipeline status, editor lists, runtime handles, and future plugin importers all need the same serialized identity instead of parallel runtime-only records.
- `ResourceRecord` exposes fluent builder helpers for artifact locator, source/importer/config hashes, state, and diagnostics. These helpers are part of the shared contract because asset import paths consume `ResourceRecord` through `zircon_runtime::core::resource`, which re-exports the interface-owned DTO.

The only helper logic allowed here is contract self-maintenance: parsing and formatting locator strings, deriving deterministic IDs, converting typed handles into untyped handles, and preserving DTO invariants. These helpers do not perform IO, spawn work, load assets, allocate runtime services, or call runtime/editor crates.

## Design And Rationale

The resource DTOs used to be compiled into `zircon_runtime_interface` through `#[path = "../../../zircon_runtime/..."]` source inclusion. That kept type identity shared, but it made the interface crate depend on runtime source layout and blurred ownership. The DTO source now lives directly under `zircon_runtime_interface/src/resource/**`, and the stale runtime-side DTO files were deleted so there is no second owner path. `zircon_runtime::core::resource` re-exports the interface contracts and keeps implementation modules such as data storage, IO, leases, registries, managers, and runtime payload state locally.

This keeps the dependency direction clean: implementation crates may depend on `zircon_runtime_interface`, but the interface crate must not depend on `zircon_runtime`, `zircon_editor`, rendering/windowing crates, dynamic loading, or system execution APIs.

## Boundary Rules

- No `#[path = ...]`, `include_str!`, or `include_bytes!` source sharing in production interface code.
- No imports from `zircon_runtime::` or `zircon_editor::`.
- No implementation-heavy dependencies such as `wgpu`, `winit`, `slint`, `libloading`, async runtimes, filesystem/network/process/thread APIs, or synchronization runtimes.
- New interface dependencies must be justified as contract or serialization dependencies and added to the boundary test allowlist deliberately.
- If behavior needs IO, scheduling, dispatch, manager access, renderer work, importer execution, or editor authoring state, it belongs outside this crate.

## Test Coverage

`zircon_runtime_interface/src/tests/contracts.rs` constructs representative resource DTOs and verifies stable locator/ID/record behavior. `zircon_runtime_interface/src/tests/boundary.rs` guards the package dependency list and scans production interface source for runtime/editor source inclusion or implementation-crate imports.

The listed Cargo commands are the intended scoped validation for this boundary. They prove the interface crate compiles, its contract tests pass, and the runtime library still type-checks while consuming the interface-owned resource DTOs. They do not claim workspace-wide acceptance.

The independent plugin manifest slice also validated this boundary with `cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --color never` after synchronizing `ResourceRecord` importer metadata with the runtime asset pipeline.

The net plugin validation slice later caught a lower-layer mismatch where the runtime asset pipeline
expected `ResourceRecord::with_state` and `ResourceRecord::with_diagnostics` through the interface
DTO. Those builders now live on `zircon_runtime_interface::resource::ResourceRecord`, matching the
canonical DTO ownership described above.
