---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/operation.rs
  - zircon_runtime_interface/src/runtime_api/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
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
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/operation.rs
  - zircon_runtime_interface/src/runtime_api/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
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
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface function_table_structs_are_all_repr_c --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface interface_public_signatures_stay_free_of_dynamic_object_exports --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface runtime_api_surface_stays_folder_backed_by_abi_owner --locked --jobs 1 --message-format short
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: module-detail
---

# Runtime API ABI Surface

`zircon_runtime_interface::runtime_api` is the stable dynamic-runtime ABI surface shared by `zircon_app`, `zircon_runtime`, editor-facing loaders, and plugin-facing hosts. The public Rust path remains `zircon_runtime_interface::runtime_api::*`; the implementation is folder-backed so new ABI families do not accumulate in one mixed file.

## Owner Split

- `runtime_api.rs` is a facade only. It declares child modules and re-exports the stable public ABI names.
- `runtime_api/api_table.rs` owns the V2 dynamic-library symbol, function pointer types, `ZrHostApiV1`, `ZrRuntimeApiV2`, and `ZrRuntimeSessionConfigV1`.
- `runtime_api/constants.rs` owns ABI numeric discriminants for event kinds, native surface kinds, window/gamepad/IME states, and fetch flags.
- `runtime_api/host_requests.rs` owns runtime-to-host request DTOs, currently IME requests and gamepad rumble requests.
- `runtime_api/plugin_event_mirror.rs` owns typed plugin-event subscription and delivery DTOs.
- `runtime_api/operation.rs` owns the runtime operation handle, submit/progress/result DTOs, terminal phase rules, and submit/poll/harvest function pointer types.
- `runtime_api/viewport.rs` owns viewport size, metrics, native surface target, and bind-surface request DTOs.
- `runtime_api/events.rs` owns `ZrRuntimeEventV1`, `ZrRuntimeTranslatedEventV1`, and event constructor helpers.
- `runtime_api/requests.rs` owns host fetch, frame capture, accessibility tree capture, and captured frame DTOs.

The V2 cutover replaces the old runtime table ABI. `zircon_runtime_get_api_v2` returns the 19-field
`ZrRuntimeApiV2`, including plugin-event subscribe/unsubscribe/drain and operation
submit/poll/harvest. Hosts resolve only V2; the old table export and loader fallback were deleted.
Existing `*V1` DTO names remain only where their payload layouts did not change.

Although the C-compatible table stores function pointers as optional slots, the V2 host contract
requires the base session, plugin-event mirror, and submit/poll/harvest operation groups. The app
loader rejects a missing member before session construction; after that gate, those accessors are
required functions rather than per-call capability fallbacks. Operation progress and result JSON
must carry `ZIRCON_RUNTIME_ABI_VERSION_V1`; the app adapter rejects a foreign DTO ABI before the
editor gateway can interpret handles, phases, operation ids, or payloads.

## Boundary Rules

The interface crate may define stable ABI records, borrowed byte-slice payloads, owned byte buffers, handles, numeric discriminants, and optional function-table slots.

It must not own runtime behavior: no `CoreRuntime`, no ECS state, no editor authoring state, no OS window objects, no GPU resources, no plugin implementation registry, and no dynamic session lifecycle. Those remain in `zircon_runtime`, `zircon_app`, or plugin runtime crates.

New ABI additions should land in the narrow owner file:

- function-table or symbol additions in `api_table.rs`;
- new event kinds or constructor helpers in `events.rs` plus `constants.rs`;
- new host request payloads in `host_requests.rs`;
- plugin-event mirror payloads in `plugin_event_mirror.rs`;
- generic long-running operation payloads in `operation.rs`;
- viewport/native surface records in `viewport.rs`;
- capture/fetch request records in `requests.rs`.

Do not add new behavior back into `runtime_api.rs`. The interface boundary test keeps this file as a small facade, requires each owner module to be declared and re-exported, and rejects oversized owner files before the ABI surface becomes another support hot spot. `tests/abi_safety_contracts.rs` additionally locks `ZrHostApiV1` and `ZrRuntimeApiV2` as `#[repr(C)]` function-table structs, fixes V2 at 19 fields, and rejects public signature lines that introduce dynamic object carriers unsuitable for the ABI boundary. The structural audit mirrors the facade shape as `runtime_api_boundary` so the owner layout is visible in architecture review output without first running the Rust test binary; `runtime_api_markdown.py` owns the audit's Markdown rendering so the boundary module remains focused on ABI shape and risk calculation.

## Validation

The split is accepted only when `zircon_runtime_interface` compiles standalone and `runtime_api_boundary` reports all eight owner modules, a small facade, no missing re-exports, no direct ABI declarations in `runtime_api.rs`, and no oversized owner modules. Focused interface tests assert the exact V2 table size/order, required mirror/operation-tail placement, event constructors, host request and plugin-event serialization, and frame/accessibility capture DTO contracts.
