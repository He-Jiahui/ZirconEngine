---
related_code:
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
implementation_files:
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/session/*.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/tests/*.rs
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Dynamic Runtime Session

`zircon_runtime::dynamic_api::session` owns the runtime session state behind the exported `zircon_runtime_interface` C ABI. The public function table remains in `dynamic_api::exports`; session code owns handle validation, session registry access, event dispatch into runtime managers, frame capture/presentation, profile control, and host request draining.

The session module is intentionally private to `zircon_runtime`. Its job is to adapt versioned ABI payloads into existing runtime facilities without turning the dynamic library boundary into a second runtime architecture.

## Owner Split

- `session.rs` keeps the FFI entry functions, session registry, `RuntimeDynamicSessionProfile`, and `RuntimeDynamicSession` lifecycle/orchestration.
- `session/status.rs` owns ABI `ZrStatus` construction for unsupported version, invalid argument, not found, and generic dynamic API errors.
- `session/host_requests.rs` converts neutral runtime input-manager host requests into ABI host request payloads.
- `session/input_events.rs` maps ABI numeric input/window/gamepad/IME constants into `core::framework::input` DTOs.
- `session/preview.rs` owns fallback frame and accessibility preview payloads used when the dynamic preview cannot extract a full UI surface.
- `tests/` mirrors the same owner split for the exported API table, profile control, viewport/frame validation, session lifecycle, host requests, accessibility, and input-event rejection paths.
- `tests/structure.rs` keeps that mirror executable by rejecting a recreated `tests.rs`, missing owner modules, non-navigational `mod.rs` content, and owner files that grow past the split threshold.

This keeps the FFI boundary file below the large-file warning line while preserving the exported `ZrRuntimeApiV1` shape.

## Boundary Rules

The dynamic session may:

- validate ABI versions and handles before touching runtime state;
- adapt host ABI values into runtime framework DTOs;
- resolve runtime managers through the existing `CoreRuntime` handle path;
- run the runtime tick, render bridge, default level, and camera-controller preview path.

The dynamic session must not:

- duplicate module assembly rules owned by `zircon_runtime::builtin::runtime_modules`;
- expose new public Rust API from the dynamic ABI internals;
- encode editor authoring state as runtime session persistence;
- bypass `core::framework` DTOs with ad hoc dynamic-only event models;
- grow conversion, status, preview, or host-request helpers back into `session.rs`.
- add new dynamic API assertions back into a monolithic `tests.rs`; new coverage belongs in the matching `tests/<owner>.rs` module.

## Validation

`zircon_runtime/src/dynamic_api/tests/` covers the exported function table, invalid ABI and handle paths, profile-control JSON validation, frame/accessibility request validation, session creation profile handling, host request encoding, accessibility fallback behavior, and input-event rejection paths.

For architecture validation, the runtime structural audit should no longer list `zircon_runtime/src/dynamic_api/session.rs` under production large-file hotspots after this split. The audit also reports `dynamic_api_test_boundary`, which must keep the legacy `zircon_runtime/src/dynamic_api/tests.rs` absent, all owner modules declared, and oversized test owner modules at zero. That audit owner now lives in `runtime_structure_audits/dynamic_api_test_boundary.py` so the main architecture audit script remains an orchestration boundary instead of becoming another mixed large file.
