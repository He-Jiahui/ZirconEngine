---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: rhi-wgpu-presenter-and-backend-contract-test-owner
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/tests
  - zircon_runtime/src/rhi_wgpu/mod.rs
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs
tests:
  - python tools/runtime_domain_dependency_audit.py --pretty
  - cargo tree -p zr_rhi --edges normal,build,dev
  - cargo test -p zr_rhi --locked
  - cargo test -p zr_rhi_wgpu --locked
  - cargo test -p zircon_runtime --lib runtime_15_rhi_command_list_tests_are_folder_backed --locked
  - cargo test -p zircon_runtime --lib runtime_15_rhi_device_contract_tests_are_folder_backed --locked
  - cargo test -p zircon_editor --lib editor_retained_host_presenter_boundary_keeps_wgpu_inside_runtime_rhi --locked
  - cargo check -p zircon_runtime --lib --locked
  - cargo check -p zircon_editor --lib --locked
---

# Frameworks01: neutral RHI owns WGPU construction and backend contract tests

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M4 production dependency and full-test-tree re-audit
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：the reverse edge cannot be removed honestly while `rhi` and `rhi_wgpu` remain source
  modules in one crate and the canonical public path must remain `zircon_runtime::rhi::*`.
  Frameworks01 M2 owns the physical `zr_rhi`/`zr_rhi_wgpu` split and facade projection.

## 失败现象与复现证据

The 2026-08-02 production dependency audit reports `rhi -> rhi_wgpu = 1`:
`zircon_runtime/src/rhi/mod.rs` implements `create_default_ui_surface_presenter` by constructing
`crate::rhi_wgpu::WgpuUiSurfacePresenter` directly. The only external caller is the Editor retained
host presenter factory, which correctly consumes the public `zircon_runtime::rhi` surface and must
not name the private backend module.

The full test tree has a second ownership problem. Twelve imports under `rhi/tests` directly use
`DeterministicRhiContractDevice`, `DeterministicRhiContractCommandList`, or WGPU backend capability
mapping from `crate::rhi_wgpu`. The backend-dependent modules are command-list/debug/device/pipeline,
render-pass, resource-lifecycle, and texture-copy contracts; only boundary, capabilities, and
descriptor tests are backend-neutral. Keeping all of them under `rhi/tests` would force `zr_rhi` to
take a dev-dependency on its WGPU implementation and invert the approved crate DAG.

The current `rhi/mod.rs`, `rhi_wgpu/mod.rs`, Editor factory, and associated test/structure-guard
paths contain foreign dirty work. Earlier successor Session registration was accepted by the
coordinator but did not materialize into a claimable Session, so Frameworks05 did not edit or move
these paths and did not create a partial compatibility projection.

## 最低共享层根因

Backend construction and the deterministic backend mirror were absorbed into the neutral RHI
contract source tree because the monolith did not yet enforce a crate boundary. The approved target
topology already provides the correct owners:

- `zr_rhi` owns only backend-neutral descriptors, capabilities, device/command traits and UI surface
  contracts;
- `zr_rhi_wgpu` depends on `zr_rhi` and owns WGPU presenter construction plus deterministic backend
  contract implementation/tests;
- the `zircon_runtime` facade owns external assembly and curated re-export. Its canonical
  `zircon_runtime::rhi::*` path may project both internal crates as an approved facade surface, but
  no internal crate may re-export the other as a migration shim.

## 架构修复验收

- During Frameworks01 M2, move neutral `rhi` production declarations to `zr_rhi` and WGPU presenter
  implementation/factory to `zr_rhi_wgpu`; `zr_rhi` must have no dependency on `zr_rhi_wgpu`.
- Preserve the canonical external `zircon_runtime::rhi::create_default_ui_surface_presenter` path
  through the facade's structural curated re-export. Remove the implementation from the neutral
  crate source; do not retain an internal `zr_rhi -> zr_rhi_wgpu` alias or wrapper.
- Delete the monolith-only `zircon_runtime/src/rhi_wgpu/` tree after its declarations move to the
  physical `zr_rhi_wgpu` crate. The facade must not retain `mod rhi_wgpu;`, and the complete Runtime
  source tree must contain zero `crate::rhi_wgpu` references. `zircon_runtime/src/rhi/mod.rs` may
  remain only as the curated public facade projection; a private legacy backend root is not an
  acceptable implementation detail.
- Keep the Editor caller backend-neutral and dependent only on `zircon_runtime` facade plus
  `UiSurfaceDescriptor`/`UiSurfacePresenter` contracts.
- Move the 12 backend-dependent test modules and their child helpers to a folder-backed
  `zr_rhi_wgpu` test owner. Keep only boundary, capabilities, and descriptors tests in `zr_rhi`.
  Delete the old `rhi/tests` backend module declarations only after every test function is mounted
  by its new owner. The flat `rhi_wgpu/tests.rs` root currently owns exactly three additional tests:
  `wgpu_caps_fall_back_to_graphics_and_copy_without_rt`,
  `native_ui_surface_source_uses_direct_surface_without_offscreen_blit`, and
  `command_copy_execution_does_not_clone_whole_source_resources`; all three must move to a
  folder-backed `zr_rhi_wgpu` test owner before the flat root is deleted. Record the pre/post test
  function inventory and require a selected count of three so deletion cannot silently reduce
  backend coverage.
- Update Runtime15 path-based file-budget/structure guards to the new physical test owners in the
  same hard cut; do not whitelist deleted paths. The focused
  `runtime_15_rhi_command_list_tests_are_folder_backed` and
  `runtime_15_rhi_device_contract_tests_are_folder_backed` guards must execute against the moved
  paths, not merely compile.
- Fresh source audit must report `rhi -> rhi_wgpu = 0`; `cargo tree -p zr_rhi` must contain neither
  `zr_rhi_wgpu` nor `wgpu`, including dev-dependencies.
- Run the Editor source guard
  `editor_retained_host_presenter_boundary_keeps_wgpu_inside_runtime_rhi`, focused neutral/backend
  tests, both Runtime15 path guards, the production dependency audit and `cargo tree`, followed by
  fresh Runtime and Editor lib checks on one immutable manifest. A compile-only Editor check does
  not prove that the backend-neutral presenter source guard executed.

## 禁止临时方案

- Do not keep `create_default_ui_surface_presenter` implemented or forwarded inside the neutral
  `zr_rhi` crate.
- Do not expose `zircon_runtime::rhi_wgpu` publicly or make Editor choose/construct the WGPU type.
- Do not leave backend test modules in `zr_rhi` behind a dev-dependency or path attribute.
- Do not add an adapter trait, duplicate presenter factory, legacy alias, or feature-specific shim.
- Do not absorb the current foreign Runtime/Editor test and profiling changes into a Frameworks05
  commit.

## 修复结果与回传

Open state: `frameworks01_m2_physical_rhi_backend_split_required`. Frameworks05 keeps the
`rhi -> rhi_wgpu` seam open and continues dependency-independent work; no RHI source move, test
migration, managed validation, fixed return, or Frameworks01 milestone acceptance is claimed here.
