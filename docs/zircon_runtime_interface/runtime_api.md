---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - zircon_runtime_interface/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
implementation_files:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime_interface runtime_api_surface_stays_folder_backed_by_abi_owner --locked --jobs 1 --message-format short
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: module-detail
---

# Runtime API ABI Surface

`zircon_runtime_interface::runtime_api` is the stable dynamic-runtime ABI surface shared by `zircon_app`, `zircon_runtime`, editor-facing loaders, and plugin-facing hosts. The public Rust path remains `zircon_runtime_interface::runtime_api::*`; the implementation is folder-backed so new ABI families do not accumulate in one mixed file.

## Owner Split

- `runtime_api.rs` is a facade only. It declares child modules and re-exports the stable public ABI names.
- `runtime_api/api_table.rs` owns the dynamic library symbol, function pointer types, `ZrHostApiV1`, `ZrRuntimeApiV1`, and `ZrRuntimeSessionConfigV1`.
- `runtime_api/constants.rs` owns ABI numeric discriminants for event kinds, native surface kinds, window/gamepad/IME states, and fetch flags.
- `runtime_api/host_requests.rs` owns runtime-to-host request DTOs, currently IME requests and gamepad rumble requests.
- `runtime_api/viewport.rs` owns viewport size, metrics, native surface target, and bind-surface request DTOs.
- `runtime_api/events.rs` owns `ZrRuntimeEventV1`, `ZrRuntimeTranslatedEventV1`, and event constructor helpers.
- `runtime_api/requests.rs` owns host fetch, frame capture, accessibility tree capture, and captured frame DTOs.

This split preserves the existing ABI and public re-export shape. It changes source ownership only.

## Boundary Rules

The interface crate may define stable ABI records, borrowed byte-slice payloads, owned byte buffers, handles, numeric discriminants, and optional function-table slots.

It must not own runtime behavior: no `CoreRuntime`, no ECS state, no editor authoring state, no OS window objects, no GPU resources, no plugin implementation registry, and no dynamic session lifecycle. Those remain in `zircon_runtime`, `zircon_app`, or plugin runtime crates.

New ABI additions should land in the narrow owner file:

- function-table or symbol additions in `api_table.rs`;
- new event kinds or constructor helpers in `events.rs` plus `constants.rs`;
- new host request payloads in `host_requests.rs`;
- viewport/native surface records in `viewport.rs`;
- capture/fetch request records in `requests.rs`.

Do not add new behavior back into `runtime_api.rs`. The interface boundary test keeps this file as a small facade, requires each owner module to be declared and re-exported, and rejects oversized owner files before the ABI surface becomes another support hot spot. The structural audit mirrors this as `runtime_api_boundary` so the facade and ABI owner shape are visible in architecture review output without first running the Rust test binary.

## Validation

The split is accepted only when `zircon_runtime_interface` compiles standalone and `runtime_api_boundary` reports 6/6 owner modules, a small facade, no missing re-exports, no direct ABI declarations in `runtime_api.rs`, and no oversized owner modules. Focused interface tests should continue to assert ABI size/order, optional function-table fields, event constructors, host request serialization, and frame/accessibility capture DTO contracts.
