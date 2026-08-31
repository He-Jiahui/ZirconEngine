---
related_code:
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/frame/frame_demand.rs
  - zircon_runtime_interface/src/runtime_api/frame/frame_shape.rs
  - zircon_runtime_interface/src/runtime_api/frame/highlight_set.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_runtime_interface/src/runtime_api/session/session_identity.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
implementation_files:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/frame/frame_demand.rs
  - zircon_runtime_interface/src/runtime_api/frame/frame_shape.rs
  - zircon_runtime_interface/src/runtime_api/frame/highlight_set.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_runtime_interface/src/runtime_api/session/session_identity.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - python -m unittest tools.tests.test_runtime_api_boundary
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface function_table_structs_are_all_repr_c --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface interface_public_signatures_stay_free_of_dynamic_object_exports --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface runtime_api_surface_stays_folder_backed_by_abi_owner --locked --jobs 1 --message-format short
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: module-detail
---

# Runtime API ABI Surface

`zircon_runtime_interface::runtime_api` is the internal lockstep dynamic-runtime ABI surface shared by `zircon_app`, `zircon_runtime`, and editor-facing loaders. It is not a published cross-version SDK. The public Rust path remains `zircon_runtime_interface::runtime_api::*`; transport-neutral world query/watch DTOs live beside it in `zircon_runtime_interface::world_sync` so the table does not own runtime behavior.

## Owner Split

- `runtime_api/mod.rs` is a facade only. It declares the `abi`, `constants`, `frame`, `host`, and `session` domains and re-exports the internal ABI names.
- `runtime_api.rs` is a deleted former owner and must not be restored beside the directory facade.
- `runtime_api/abi/api_table.rs` owns V7 function pointer types, `ZrHostApiV1`, and `ZrRuntimeApiV7`; `abi/{api_shape,host_api_shape}.rs` own copied-table validation. The frozen InterfaceSpec generates the V7 entry symbol and table-version constant through the separate `runtime_build_set/` deployment-identity domain.
- `runtime_api/constants.rs` owns ABI numeric discriminants for event kinds, native surface kinds, window/gamepad/IME states, and fetch flags.
- `runtime_api/frame/{frame_demand,frame_shape,highlight_set}.rs` own frame scheduling, frame/capture DTOs, and highlight payloads.
- `runtime_api/host/host_requests.rs` owns runtime-to-host request DTOs, currently IME requests and gamepad rumble requests.
- `runtime_api/session/session.rs` owns `ZrRuntimeSessionConfigV3`; `session/{events,requests,viewport}.rs` own session event, capture/fetch, and viewport/native-surface DTOs.
- `runtime_api/session/plugin_event_mirror.rs` owns typed plugin-event subscription and delivery DTOs; `session/operation.rs` owns operation handles, submit/progress/result DTOs, terminal phase rules, and submit/poll/harvest function pointer types.
- `runtime_api/session/session_identity.rs` owns the Rust-only gateway session identity used by the loader/session protocol.
- `buffer.rs` owns byte carriers plus the shared byte/item/depth/time budgets used by runtime producers and host consumers.

The V7 cutover replaces all older runtime tables. `zircon_runtime_get_api_v7` returns the frozen
25-field `ZrRuntimeApiV7`, including one allocation-release entry plus plugin-event, operation, and required world
query/watch/unwatch/drain entries. Hosts resolve only V7; old table exports and loader fallbacks
are deleted. Existing `*V1` DTO names remain only where their payload layouts did not change.

`ZrRuntimeSessionConfigV3` carries the selected profile, one physical `project_root` anchor, an
optional project-relative `play_scene`, and an optional logical `play_report_pipe` outlet. The
runtime validates and resolves those inputs before it creates a session; a versioned Play scene is
decoded before the first runtime frame, not by rewriting the project manifest or replacing a
default world after startup.

Although the C-compatible table stores function pointers as optional slots, the V7 host contract
requires allocation release plus the base session, plugin-event mirror, operation, and world-sync groups. The app loader
rejects a missing member before session construction; after that gate, those accessors are required
functions rather than per-call capability fallbacks. World query and invalidation payloads are JSON
inside immutable `ZrOwnedResultV2` records; watches use the transparent, runtime-issued `WatchToken`, never an
editor view id.

Runtime-owned output never exposes allocator capacity, an owner token, or a per-result free callback.
The producer registers each non-empty result under an opaque `ZrRuntimeAllocationId`; hosts borrow
the immutable bytes and release the originating session handle plus that id through
`ZrRuntimeApiV7::release_allocation`. Duplicate, forged, wrong-session, and concurrent losing
releases return `NotFound` without removing storage or changing its owner census. Session destruction is rejected while its
allocation census is nonzero and may be retried after the outstanding results are released.

Borrowed `ZrByteSlice` values must cross `checked_slice` before Rust constructs a slice. The check
rejects null plus nonzero length, lengths above `isize::MAX`, and lengths above the selected call
family's byte budget. Empty slices remain valid where the call contract allows them. JSON request
and result families share `ZrRuntimePayloadLimitV1` values for encoded bytes, structural items,
nesting depth, and processing time; producer and host policy constants derive from the same values.
Malformed carriers or JSON return `InvalidArgument`, while an otherwise shaped request or output
that exceeds a resource budget returns `LimitExceeded`.

Frame requests are rejected before rendering when either dimension exceeds 16,384 or the computed
RGBA footprint exceeds 256 MiB. Runtime status diagnostics use a 4 KiB UTF-8-aligned thread-local
buffer instead of leaking an allocation. A diagnostics pointer is valid only until the next dynamic
status is produced on the same thread, so consumers must inspect it synchronously and enforce the
same 4 KiB limit.

## Boundary Rules

The interface crate may define stable ABI records, checked borrowed byte-slice payloads, immutable owned-result views, opaque handles, resource-budget constants, numeric discriminants, and optional function-table slots.

It must not own runtime behavior: no `CoreRuntime`, no ECS state, no editor authoring state, no OS window objects, no GPU resources, no plugin implementation registry, and no dynamic session lifecycle. Those remain in `zircon_runtime`, `zircon_app`, or plugin runtime crates.

New ABI additions should land in the narrow owner file:

- function-table type additions in `abi/api_table.rs`; entry-symbol, table-version, and slot-inventory changes in the frozen `runtime_build_set` InterfaceSpec and its generator;
- new event kinds or constructor helpers in `session/events.rs` plus `constants.rs`;
- new host request payloads in `host/host_requests.rs`;
- plugin-event mirror payloads in `session/plugin_event_mirror.rs`;
- generic long-running operation payloads in `session/operation.rs`;
- transport-neutral world query, watch, and invalidation DTOs in `world_sync/` with their ABI
  function pointers in `abi/api_table.rs`;
- viewport/native surface records in `session/viewport.rs`;
- capture/fetch request records in `session/requests.rs`.

Do not add new behavior back into `runtime_api/mod.rs`. The interface boundary test keeps this file as a small facade, requires each owner module to be declared and re-exported, and rejects oversized owner files before the ABI surface becomes another support hot spot. `tests/abi_safety_contracts.rs` locks `ZrHostApiV1` and `ZrRuntimeApiV7` as `#[repr(C)]` function-table structs, fixes V7 at 25 fields and its complete field order, and rejects public signature lines that introduce dynamic object carriers unsuitable for the ABI boundary. The structural audit mirrors the facade shape as `runtime_api_boundary` so the owner layout is visible in architecture review output without first running the Rust test binary; `runtime_api_markdown.py` owns the audit's Markdown rendering so the boundary module remains focused on ABI shape and risk calculation.

## Validation

The split is accepted only when `zircon_runtime_interface` compiles standalone and `runtime_api_boundary` reports the declared owner modules, a small facade, no missing re-exports, no direct ABI declarations in `runtime_api/mod.rs`, and no oversized owner modules. Focused interface tests assert the exact V7 table size/order, mandatory allocation release, required mirror/operation/world-sync placement, event constructors, host request and plugin-event serialization, world-sync JSON/token contracts, and frame/accessibility capture DTO contracts.
