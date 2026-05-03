---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/assets/ui/runtime/fixtures
implementation_files:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/tests/graphics_surface/mod.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
plan_sources:
  - user: 2026-04-20 implement the workspace hard cutover and standardize the result
  - .codex/plans/ZirconEngine 全仓结构硬切换与规范固化计划.md
tests:
  - zircon_runtime/src/tests/graphics_surface/mod.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/graphics/tests/boundary.rs
doc_type: module-detail
---

# Runtime Surface And Assets Rules

## Purpose

This document captures the runtime-side structure rules introduced by the workspace hard cutover: narrow public surfaces, no migration-smell directories, and crate-owned runtime assets.

## Graphics Surface Rules

- `zircon_runtime::graphics` exposes only stable runtime-facing contracts.
- Deep renderer construction helpers, overlay icon seams, and viewport frame assembly stay internal or test-only.
- Empty or dead migration-smell directories such as `graphics/compat` and `graphics/service` do not survive the cutover.

## Runtime UI Asset Rules

- Production runtime `.ui.toml` resources live under `zircon_runtime/assets/ui/runtime/fixtures/`.
- Runtime code must load fixture assets from crate `assets/`, not from `src/`.
- Any old `src/.../fixtures` path or compatibility branch is migration debt.

## Test Tree Rules

- Root runtime tests are grouped by responsibility instead of growing one umbrella surface.
- Folder-backed test owners such as `src/tests/ui_boundary/` keep `mod.rs` structural only; runtime UI asset, runtime-host, namespace-boundary, and absorption assertions live in focused child files.
- Boundary assertions should check public-surface narrowing, runtime asset loading, and owner-path convergence.

## Crate Root Rules

- `zircon_runtime/src/lib.rs` stays a structural entry and registration surface.
- Plugin package manifests, export-plan DTOs, native plugin ABI/load types, runtime extension registries, runtime plugin catalogs, and plugin UI/component descriptors are owned by `zircon_runtime::plugin`; callers must import them from that namespace instead of the `zircon_runtime` crate root.
- The workspace root manifest keeps only dependencies still used by root members; generated Slint build seams such as `slint-build` are not retained after the Rust-owned host cutover.
- `graphics/mod.rs` stays a narrow runtime-facing export layer, not a deep implementation barrel.
- Internal convenience flattening is tolerated only when it stays crate-private and does not re-create a public compatibility surface.
