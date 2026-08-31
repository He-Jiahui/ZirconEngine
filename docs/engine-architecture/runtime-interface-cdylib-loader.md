---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/window_creation.rs
  - zircon_app/src/entry/runtime_library/mod.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/plugin_events.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_editor/src
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/window_creation.rs
  - zircon_app/src/entry/runtime_library/mod.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/plugin_events.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_editor/src
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
plan_sources:
  - user: 2026-05-01 request runtime/editor/plugin compile isolation through interface crate plus runtime cdylib
  - docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md
  - docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md
  - docs/superpowers/plans/2026-05-10-runtime-surface-present.md
  - docs/superpowers/plans/2026-05-04-sound-dynamic-event-execution.md
tests:
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/tests/mod.rs
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
  - cargo fmt --all --check
  - cargo test -p zircon_runtime_interface --locked --verbose
  - cargo test -p zircon_app --locked --verbose
  - cargo test -p zircon_app --locked --verbose (2026-05-11 blocker rerun: passed once after Cargo generated unrelated `taffy` lock entries; final clean-lock rerun is blocked before compile by the unrelated `zircon_runtime/Cargo.toml` / `Cargo.lock` mismatch)
  - cargo test -p zircon_runtime_interface plugin_event --locked --offline --jobs 1
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
- `buffer.rs` defines borrowed byte slices, plugin-owned callback buffers, and immutable runtime results released by opaque allocation id.
- `runtime_api/mod.rs` is the structural facade over the `runtime_api/{abi,constants,frame,host,session}` owner domains. The folder defines the frozen V7 runtime function-table shape and symbol; `world_sync/**` owns transport-neutral query, watch, token, and invalidation DTOs. Together they define fixed event records, viewport sizing records, native surface binding requests, runtime-to-host request DTOs, plugin-event and operation lifecycle DTOs, frame/accessibility capture requests, and typed captured-frame results.
- `plugin_api.rs` defines the plugin entry symbol, v1 plugin entry report shape, and optional plugin-side callback slots.
- `plugin_events.rs` defines the generic v1 plugin event callback ABI. Subsystems such as sound project their own neutral event DTOs into namespace-tagged byte-slice requests instead of passing Rust trait objects or subsystem-owned runtime state across dynamic-library boundaries.
- `manifest.rs` defines target mode, module kind, and module descriptor DTO seeds for later runtime/plugin adapters.
- `ui/mod.rs` exposes the shared neutral Runtime UI contract namespace for editor-facing UI DTOs: `binding`, `component`, `dispatch`, `event_ui`, `layout`, `surface`, `template`, and `tree`. The UI namespace is now backed by real `zircon_runtime_interface/src/ui/**` files instead of path-including `zircon_runtime/src/ui/**`. Runtime-only behavior such as component registries, event managers, dispatchers, layout passes, render extraction, text layout, tree mutation, template loading, compiling, and validation remains owned by `zircon_runtime`.

## Milestone 1 Runtime Cdylib

`zircon_runtime` declares `crate-type = ["rlib", "cdylib"]` and exposes only
`zircon_runtime_get_api_v7` from `zircon_runtime::dynamic_api`. It returns the frozen 25-field
`ZrRuntimeApiV7` after validating `ZrHostApiV1`; retired runtime tables and entry symbols are not
exported or aliased.

Runtime 10 M1.3 owns the final panic containment layer at that dynamic-library edge. The V7 table-acquisition export returns a null table pointer if acquisition unexpectedly unwinds, and every advertised V7 function pointer targets an `exports.rs` `_ffi` wrapper that converts unexpected unwinds to `ZrStatusCode::Panic` instead of letting panic state cross the C ABI. The private `dynamic_api::session` and `session::operation` owner functions stay Rust-ABI `unsafe fn` so the wrapper can catch an unwind before it reaches the exported C ABI edge.

The dynamic runtime session owns the concrete runtime implementation objects that previously lived in `zircon_app` runtime preview code:

- `CoreRuntime` and activated target-client runtime modules.
- Runtime-owned `Time<Real>`, `Time<Virtual>`, `Time<Fixed>`, and diagnostic store state advanced through the optional `tick_frame` ABI.
- Default level/world state and selected-node orbit target.
- Runtime camera interaction state.
- Input manager event routing.
- Render framework viewport creation, extract submission, and captured-frame retrieval.

`ZrRuntimeSessionConfigV3.profile` is interpreted at session creation before runtime bootstrap. Empty and `runtime` create the normal runtime preview profile, while `editor`, `dev`, `minimal`, and `headless` are accepted named profiles for host-specific policy. Unknown profile bytes are rejected as invalid arguments before allocating a dynamic session, so dev-profile behavior branches on a stable profile enum rather than string checks scattered through the runtime. The same carrier holds one resolved physical `project_root` anchor plus optional project-relative `play_scene` and logical `play_report_pipe` startup inputs. The runtime resolves the relative scene under that root and materializes it before first-frame selection; it does not modify the project manifest. The current `dev` dynamic session enables Bevy `LogDiagnosticsPlugin`-style diagnostic-store logging on a one-second runtime-owned schedule; the app host still only calls `tick_frame` and does not inspect runtime diagnostics.

The ABI boundary receives only `ZrRuntimeEventV1` values for viewport resize, pointer motion, mouse button, mouse wheel, keyboard, IME, touch, gamepad, lifecycle, accessibility action, and raw mouse motion input. It returns `ZrRuntimeFrameV2`, whose immutable `rgba` result carries only a pointer, fixed-width length, and opaque allocation id. The host releases that id together with its originating session handle through the mandatory V7 `release_allocation` entry; it never reconstructs a runtime allocator object from caller-writable metadata. The registry checks the session owner before removal, so a crossed-session id cannot free another live session's bytes or reduce its teardown census.

## Milestone 1 App Loader

`zircon_app` runtime profile now loads runtime with `libloading` instead of bootstrapping runtime preview internals directly. The loader resolves the library path from `ZIRCON_RUNTIME_LIBRARY` first, then falls back to an executable-sibling platform name: `zircon_runtime.dll`, `libzircon_runtime.dylib`, or `libzircon_runtime.so`. Development builds launched directly from Cargo target directories also check executable-sibling `deps/<platform runtime library>` when the packaged sibling library has not been staged yet.

The runtime runner accepts an explicit dynamic session policy argument before the library session is created: `--runtime-session-profile <runtime|editor|dev|minimal|headless>` or `--runtime-session-profile=dev`. The runner strips diagnostic log arguments first, parses the session profile second, rejects duplicate or unknown profile arguments, and passes the selected bytes through `RuntimeSession::create_with_profile(...)` into `ZrRuntimeSessionConfigV3.profile`. `-h` and `--help` print the accepted profile names, log controls, and `ZIRCON_RUNTIME_LIBRARY` override before loading the dynamic runtime library. This is intentionally narrower than Bevy's code-level plugin group selection: Zircon's app host chooses the dynamic runtime session policy at process startup, while concrete module ownership, runtime clocks, and dev diagnostic cadence stay inside `zircon_runtime`.

`RuntimeEntryApp` now owns only the window, optional softbuffer presenter, dynamic runtime session wrapper, viewport handle, and current viewport size. Winit events are converted to interface events and sent to runtime. `about_to_wait` calls the optional `tick_frame` ABI before requesting redraw, so the dynamic runtime's `CoreRuntime::tick_time(...)` path advances before the next present or frame capture. Redraw requests prefer the optional runtime surface-present ABI only when `bind_viewport_surface`, `unbind_viewport_surface`, and `present_viewport` are non-null in the exact frozen V7 table. Otherwise, redraw falls back to `capture_frame` and blits the returned RGBA bytes through softbuffer.

`RuntimeSession` calls `destroy_session` before it releases the host wake registration or lets
`LoadedRuntime` drop its dynamic library. A non-OK destroy result is an unrecoverable process
boundary failure: it means the runtime cannot prove its copied host callback and DLL-owned workers
have stopped. The host records the diagnostic and terminates the process before normal Rust drop
can unload the library. It does not leak or retain `libloading::Library` as a recovery mechanism.
The runtime rejects synchronous same-session destroy from inside that session's active wake callback
before close admission, because waiting for callback quiescence there would wait on the caller's own
in-flight guard. A destroy issued from another thread retains the normal disable-and-drain barrier.
The same fail-fast rule applies when `create_session` fails after acquiring its dynamic diagnostic
worker lease: it must either join that worker before returning the construction error or terminate
before a caller can unload the library.

`LoadedRuntime` resolves only V7. A missing symbol or invalid table is rejected without downgrade.
The loader requires both `abi_version == ZIRCON_RUNTIME_API_VERSION_V7` and
`size_bytes == size_of::<ZrRuntimeApiV7>()`; shorter prefixes and oversized same-version tables are
rejected because the V7 shape is frozen. Required slots must be non-null, while optional slots are
represented only by a null function pointer inside that exact layout. The editor product may use the same
loader with the process-linked V7 table so linked plugin registration reports and
the runtime session registry remain in one image. `RuntimeEntryApp` binds native Win32 surface
metadata before the resize event when the coherent surface path is available, rebinds before later
resize events, unbinds before falling back if a previously enabled surface-present path stops
presenting, and best-effort unbinds during `Drop` before the window and softbuffer presenter fields
are destroyed.

Runtime 10 M3 tightens the cdylib failure paths around this loader. `LoadedRuntime::load` owns the
real `libloading` open and V7-only lookup, while `validate_runtime_api_pointer(...)` tests table
rejection without constructing a real dynamic library. Focused tests cover null, wrong table
version, short or oversized frozen layouts, missing required functions, incomplete V7
mirror/operation/world-sync lifecycle, and the real V7-symbol-missing branch. Source guards also lock the single symbol lookup and the first `create_session` call's `create runtime session`
failure context.

Once V7 validation succeeds, `LoadedRuntime` exposes base session, plugin-event mirror,
submit/poll/harvest, and world-sync entries as required functions. `RuntimeSession` therefore has no secondary
"capability unavailable" branch for these groups. Poll and harvest decode the runtime-owned JSON
buffer, free it through its callback, then require `ZIRCON_RUNTIME_ABI_VERSION_V1` before returning
the typed DTO. Editor operation commands repeat the ABI check at the command boundary so alternate
gateway implementations cannot inject a foreign progress/result layout into transaction logic.

### Current V7 Structural Evidence

The V7 source inventory has `expected_source_file_count = 60`, including World Sync DTO,
allocation registry, host output, and editor gateway owners. The `dynamic_runtime_api_boundary`
guard pins `function_table_structs = 12/12`, the 25-field `ZrRuntimeApiV7` table, and
`runtime_session_ffi_wrappers = 23/23`; Cargo acceptance remains separately recorded.
The permanent guard remains `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts`.

## Validation

Milestone 0 validation is intentionally scoped to the interface crate. The required checks prove the crate compiles by itself, its contract tests pass, and it does not pull in `zircon_runtime`, `zircon_editor`, `wgpu`, `slint`, or plugin implementation crates through dependencies.

Milestone 1 validation adds runtime library build coverage, app target-client checking, and scoped dynamic API/runtime-loader tests. The validation must prove the cdylib export is available, the app runtime profile uses the interface table, and app runtime preview source no longer imports runtime implementation preview objects.

The Bevy-style time continuation adds focused coverage for the `tick_frame` field: interface contract tests verify field size and ordering after `profile_control`; dynamic API tests verify export presence plus unknown-session and valid-session behavior; app runtime-library tests verify a null slot is treated as optional only after the complete frozen V7 layout is accepted; and app entry source guards verify `about_to_wait` advances dynamic runtime time before `request_redraw`. Dynamic API tests also cover session-profile parsing by rejecting unknown profile bytes before runtime bootstrap, accepting the named `dev` profile, and guarding that the dev profile ticks a `DiagnosticStoreLogSchedule` before writing `collect_runtime_diagnostics(...).store` through `write_diagnostic_store_snapshot`. App-side parser tests and entry source guards cover `--runtime-session-profile` stripping, duplicate/missing/unknown argument rejection, help output coverage, and forwarding through `RuntimeSession::create_with_profile(...)`.

Runtime 10 loader failure validation adds focused app-library tests for `validate_runtime_api_pointer(...)`, plus source guards for missing symbol and first-call failure context. The scoped checks passed on 2026-06-12: `runtime_api_pointer_rejects_*` passed 3/3, `runtime_library_loader_reports_missing_entry_symbol_source_path` passed 1/1, `runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library` passed 1/1, and `runtime_session_create_reports_first_call_failure_context` passed 1/1 under `cargo test -p zircon_app --lib ... --locked`. Full `cargo test -p zircon_app --locked` remains pending.

Runtime 10 FFI panic-boundary validation is currently source-guarded by `runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary`, which requires all 11 runtime session table entries to point at `exports.rs` wrappers and requires the shared `ZrStatusCode::Panic` diagnostic path. Cargo execution for the dynamic API filter remains pending while runtime compile lanes are active.

`dynamic_runtime_api_boundary` now mirrors the Runtime 10 dynamic runtime API ABI/session/loader/host-request boundary in the structural audit, with Markdown rendering split into `dynamic_runtime_api_markdown.py`. Current static evidence reports `dynamic_runtime_api_boundary.py` at 330 audit/risk lines, the Markdown owner at 65 lines, `expected_source_file_count = 35`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `runtime_diagnostics_anchors = 15/15`, `missing_runtime_diagnostics_anchors = []`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_contract_duplicate_public_types = 0`, `ui_v2_contract_sync_anchors = 9/9`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, and `risks = []`. The host-request payload inventory keeps the dynamic-library return path pinned from `ZrRuntimeHostRequestBatchV1` through runtime conversion and app-side routing for IME, gamepad rumble, and cursor requests. Runtime diagnostics now travel through the existing `profile_control` JSON ABI via `ProfileControlCommand::RuntimeDiagnosticsSnapshot`, with no new `ZrRuntimeApiV1` function pointer. `runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending` records that runtime-local `UiBindingCodec` and `UiAssetSchemaVersionPolicy` duplicates were removed while interface-owned UI contracts remain the shared boundary. `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending` records that Runtime 09's `v2-replacement-mainline` verdict is synchronized into the interface/runtime split, with `UiComponentApiVersion` kept as the shared version contract. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps this loader doc aligned with the dynamic API module doc, Runtime 10, the runtime index, the M0 review, and runtime-interface convergence. This is static evidence only; the `dynamic_api`, full app loader, and UI contract Cargo lanes remain pending.

The 2026-06-20 diagnostics inventory split records `runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_diagnostics_inventory.py` now owns Runtime 10 diagnostic anchor tuples, and the loader mirror includes `scene_asset_reload_diagnostic_path_anchors = 21/21` plus `missing_scene_asset_reload_diagnostic_path_anchors = []` next to `runtime_diagnostics_anchors = 15/15`.

The 2026-06-20 host-request inventory split records `runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_host_request_inventory.py` now owns the 38 host-request payload anchor tuples, while the loader mirror still requires `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, and `risks = []`.

The 2026-06-20 UI contract inventory split records `runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_ui_contract_inventory.py` now owns the UI pending-gate, single-source contract, and v2 sync anchor tuples, while the loader mirror still requires `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_v2_contract_sync_anchors = 9/9`, and `risks = []`.

The 2026-06-20 validation inventory split records `runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_validation_inventory.py` now owns the behavior-test, pending Cargo gate, doc-anchor, and mirror-doc guard tuples, while the loader mirror still requires `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `missing_doc_anchors = []`, and `risks = []`.

The 2026-06-20 session lifecycle inventory split records `runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_session_lifecycle_inventory.py` now owns the headless/minimal lifecycle anchor tuples, while the loader mirror still requires `headless_lifecycle_anchors = 12/12`, `missing_headless_lifecycle_anchors = []`, and `risks = []`.

The 2026-06-20 failure boundary inventory split records `runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_failure_inventory.py` now owns the FFI panic and loader failure anchor tuples, while the loader mirror still requires `ffi_panic_anchors = 9/9`, `missing_ffi_panic_anchors = []`, `loader_failure_anchors = 10/10`, `missing_loader_failure_anchors = []`, and `risks = []`.

The 2026-06-21 ABI source inventory split records `runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred`: `dynamic_runtime_api_abi_inventory.py` now owns the source owner, function-table shape, and session operation tuples, while the loader mirror still requires source files 35/35, function tables 10/10, runtime session wrappers 11/11, no direct table-entry bypass, `session_owner_extern_c_present = false`, and `risks = []`. The focused package check for `cargo test -p zircon_runtime --lib dynamic_api_session` timed out after 904s with no test result, so loader/app gates remain pending.

Milestone 2 first-slice validation is scoped to the shared UI contract namespace and editor library type checking. The interface crate check proves the real interface-owned UI contract modules compile without depending on `zircon_runtime`, `zircon_editor`, Slint, wgpu, or plugin crates. The editor library check proves the current editor UI host can type-check after the interface tree split, but it does not prove the editor import cutover is complete: a 2026-05-02 audit found 134 `zircon_runtime::ui` hits and 431 `zircon_runtime_interface::ui` hits in `zircon_editor/src`. The residual runtime hits must be split by role: neutral DTOs should move to `zircon_runtime_interface::ui`, while concrete services such as `UiSurface`, `UiEventManager`, `UiDocumentCompiler`, `UiAssetLoader`, `UiTemplateSurfaceBuilder`, `UiTemplateBuildError`, `UiComponentDescriptorRegistry`, `UiAssetDocumentRuntimeExt`, and `UiPointerDispatcher` remain runtime behavior dependencies. An earlier `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover-opencode --message-format short --color never` passed with existing warnings, and the 2026-05-02 19:44 current-worktree rerun `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never` also passed with existing runtime graphics warnings and 3 editor warnings. `cargo tree -p zircon_editor --locked --depth 1` still lists direct `zircon_runtime` and `zircon_runtime_interface` dependencies for the documented service/contract split.

The `UiSurface` and `UiTree` storage identity has now converged. `zircon_runtime_interface::ui::tree` owns serializable tree contract DTOs, and `zircon_runtime::ui::surface::UiSurface` stores the interface `UiTree` directly. Runtime still owns insertion, mutation, focus, hit-test, scroll, render-order, and routing behavior through `zircon_runtime::ui::tree::UiRuntimeTree*Ext` traits and helper services, so editor surface builders import tree DTOs from the interface crate and import runtime extension traits only when they call behavior methods.

The 2026-05-24 plugin-event ABI slice was validated with scoped interface evidence: `rustfmt --edition 2021 --check zircon_runtime_interface\src\plugin_events.rs zircon_runtime_interface\src\plugin_api.rs zircon_runtime_interface\src\lib.rs zircon_runtime_interface\src\tests\contracts.rs ...` passed for the interface files, `cargo fmt --check --manifest-path Cargo.toml -p zircon_runtime_interface` passed, `cargo metadata --manifest-path zircon_runtime_interface\Cargo.toml --locked --offline --no-deps --format-version 1` passed, and `cargo test -p zircon_runtime_interface plugin_ --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-sound-dynamic-event-abi-interface --message-format short --color never` passed with 2 tests and 110 filtered out.

## Plugin Event Callback ABI

`ZrPluginApiV1` now reserves an optional trailing `invoke_event` function pointer. The field is appended after `unload`, and contract tests pin the table size and offset on the current target so existing prefix readers can continue to gate optional slots by `size_bytes`.

`ZrPluginEventCallbackRequestV1` is intentionally generic. It carries an ABI version, subsystem namespace, plugin ID, handler ID, event ID, optional source path, event time, payload schema, and opaque payload bytes as borrowed `ZrByteSlice` values. The callback writes `ZrPluginEventCallbackResultV1`, whose status and diagnostics capture handler-level acceptance or rejection without letting a subsystem pass Rust closures, runtime managers, scene state, or editor objects across the boundary.

The first concrete consumer is the sound runtime adapter: sound projects `SoundDynamicEventDelivery` into the generic callback request under the `sound.dynamic_events` namespace. The interface crate does not know sound semantics, and the sound framework DTOs still do not own dynamic-library function pointers. Generic native plugin discovery and attachment of `ZrPluginApiV1::invoke_event` to subsystem handler descriptors remains loader/runtime integration work outside this interface DTO slice.
<!-- Runtime 10 V7-only audit mirror: expected_source_file_count = 60; runtime_session_ffi_wrappers = 23/23 -->
