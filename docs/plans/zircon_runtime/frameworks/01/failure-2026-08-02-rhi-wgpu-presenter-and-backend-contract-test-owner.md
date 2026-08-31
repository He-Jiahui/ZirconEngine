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
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
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

Resolving state: `frameworks01_m2_physical_rhi_backend_split_implemented_validation_pending`.

- 2026-08-03: created physical workspace members `zr_rhi` and `zr_rhi_wgpu` under
  `zircon_runtime/crates/`. `zr_rhi` owns neutral descriptors, handles, device/command contracts,
  native surface target and UI surface contracts; its manifest contains no WGPU dependency. Runtime
  compiles this lightweight contract crate unconditionally because the always-on `RenderFramework`
  trait names its native surface target, while only `zr_rhi_wgpu` remains graphics-optional.
- `zr_rhi_wgpu` now depends downward on `zr_rhi`, owns the WGPU presenter plus shared GPU timer and
  readback implementation, and exposes only the backend primitives consumed by Runtime graphics.
  The Runtime facade keeps the canonical `zircon_runtime::rhi::*` projection and the single default
  presenter factory; the neutral crate does not construct or re-export the backend.
- Deleted both monolithic source directories (`zircon_runtime/src/rhi/` and
  `zircon_runtime/src/rhi_wgpu/`). Current production source has zero `crate::rhi_wgpu` references
  and `zircon_runtime/src/lib.rs` has no `mod rhi_wgpu` declaration.
- Moved all 12 deterministic-backend test modules and their child owners to folder-backed
  `zr_rhi_wgpu/src/tests/`; `zr_rhi` retains only boundary, capabilities, descriptors and neutral UI
  tests. The three former flat backend-root tests are mounted from
  `zr_rhi_wgpu/src/tests/capabilities.rs` and the existing backend child modules.
- Updated Runtime15 path guards, server feature-boundary checks and dependency-domain inventory to
  the physical paths. Non-Cargo evidence is GREEN: Rust 1.94.1 exact-scope `rustfmt` passed,
  `python -m unittest tools.tests.test_frameworks_03_server_feature_boundary` passed 13/13, and
  final independent re-review reran `python tools/runtime_domain_dependency_audit.py --repo-root .`
  with 2,586 production references / 70 domain edges and no Runtime `rhi_wgpu` domain.
- The first four independent review passes reported `C0/I3/M1`, `C0/I1/M1`, `C0/I1/M0` and
  `C0/I1/M0`: they
  found stale backend `include_str!` depths, two deleted-path Runtime15 guards, an incomplete
  reverse-dependency guard, a dotted-key dependency-detector gap, missing non-Cargo coverage for
  that detector, and stale executable documentation owners. Every finding is repaired; all audited
  literal and manifest-anchored include targets resolve, deleted Runtime test-reader scans are
  absent, and obsolete current owner paths and executable documentation commands are removed. The
  final independent re-review returned `C0/I0/M0`; its fresh dependency audit confirmed 2,586
  production references / 70 domain edges, `asset -> text = 0`, `rhi -> rhi_wgpu = 0`, and only the
  separately owned `scene -> animation = 2` reverse seam remains.
- Locked metadata and the read-only neutral `cargo tree` are GREEN. Coordinator snapshot `1462`
  accepted the implementation manifest, and validation-copy request
  `bb3618ddccc74633b654c29cf727df38` created managed job
  `d863427f3d904bde8ad07e09a050c3d0` for the Rust 1.94.1 `zr_rhi`/`zr_rhi_wgpu` focused test batch.
  This is an accepted receipt only, not a GREEN result; the status-only record correction on this
  file occurred after that immutable validation copy and therefore requires a refreshed commit
  snapshot. Runtime/Editor guards and immutable Runtime/Editor lib checks also remain pending.
  Therefore this failure is not yet `fixed`, no fixed return is emitted, and Frameworks01 M2 is not
  accepted.

## 2026-08-22 current-source physical-split re-audit

- On current HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`, the production dependency audit
  reports 2,749 references / 72 domain edges and still reports `rhi -> rhi_wgpu = 0`. A second scan
  over 4,938 existing non-test Runtime Rust files reports zero `crate::rhi_wgpu` or
  `mod rhi_wgpu` references. The deleted `zircon_runtime/src/rhi/` and
  `zircon_runtime/src/rhi_wgpu/` directories remain absent.
- Read-only `cargo tree -p zr_rhi --edges normal,build,dev --locked` completed with 23 output lines
  and zero `wgpu`/`zr_rhi_wgpu` dependencies. The neutral manifest likewise names neither backend;
  `zircon_runtime/src/rhi.rs` remains the approved curated `zr_rhi` projection and the facade-owned
  default WGPU presenter assembly point rather than an internal compatibility bridge.
- All three migration-sensitive backend tests remain mounted in
  `zr_rhi_wgpu/src/tests/capabilities.rs`: `wgpu_caps_fall_back_to_graphics_and_copy_without_rt`,
  `native_ui_surface_source_uses_direct_surface_without_offscreen_blit`, and
  `command_copy_execution_does_not_clone_whole_source_resources` (selected count 3/3).
- These are current-source static/read-only gates only. Fresh managed `zr_rhi`/`zr_rhi_wgpu`,
  Runtime15, Runtime lib and Editor lib gates plus a current immutable review are still required;
  the Failure stays `open`, no fixed return is emitted, and Frameworks01 M2 remains unaccepted.
- The local Unreal module re-check gives the same dependency direction: `D3D12RHI.Build.cs` and
  `VulkanRHI.Build.cs` depend on the public `RHI` module, while `RHI.Build.cs` selects concrete
  backends through dynamically loaded modules. This supports Zircon's `zr_rhi <- zr_rhi_wgpu`
  contract plus Runtime-facade backend assembly; it does not justify a neutral-crate dependency on
  WGPU or an Editor-owned backend factory.
- On successor base `b674450632e152ef265e7f6d0fcca93d978e814d`, formal neutral-crate copy
  `640404a486f54937bc2ea1bfef121485` fixed external source
  `E:\Git\zr_vm@f845719fe337136bbea8a94d730eca92d420fe00` (source hash
  `038d971e7623198ddb462a31c434f8ee0c0fe1d74224b68724363cd2cde8821c`) and requested exact
  `cargo test -p zr_rhi --locked --jobs 1`. Workspace closure preparation still reached five
  un-attributed Runtime74 UI blobs and terminated with `validation_copy_baseline_drift` before an
  input manifest or Cargo process existed. The F-drive copy was cleaned; this is scheduling/source
  attribution evidence, not a neutral RHI RED, and the required test gate remains pending.

## 2026-08-23 r9 current-source feature-boundary confirmation

- On current HEAD `f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`, fresh command
  `python -B -m unittest tools.tests.test_frameworks_03_server_feature_boundary -v` is GREEN 14/14.
  The tests themselves completed in 2.126 seconds and the PowerShell command completed in 3.35
  seconds with `TEMP`/`TMP` routed to the repository E-drive coordinator state directory.
- This refresh covers optional graphics/backend dependencies, physical `zr_rhi`/`zr_rhi_wgpu`
  feature closure, dotted-key reverse-dependency detection, server exclusion of client domains, and
  root-domain cfg gates. It is source-shape evidence only: managed `zr_rhi`/`zr_rhi_wgpu`, Runtime,
  Editor, current immutable review, fixed return, and coordinator commit remain pending. The Failure
  stays `open`, and Frameworks01 M2 is not accepted.
- Current exact4 materialization request `182160dcea314a93b6715971cd59b21b` created D/F-drive copy
  job `e5eba1bb5e124633a477c3e4613fcdf1` for
  `cargo +1.94.1 test -p zr_rhi --locked --jobs 1 --color never`. It pinned external source
  `E:\Git\zr_vm@bad3722bce67d9bf15d2109f7eca64f234c1a2ee` with only the binding and sys crate
  roots. The copy stopped at `materialization_prepare` with
  `validation_copy_baseline_drift` on the same five Runtime74-owned files:
  `ui/surface/binding_targets.rs`,
  `ui/template/asset/compiler/{binding_param_resolver,control_scope}.rs`, and
  `ui/tests/{asset_binding,asset_prototype_store}/control_scope.rs`.
  No input manifest or Cargo process was created. This is current source-closure blocking evidence,
  not a neutral-RHI test failure; Frameworks01 does not absorb, reattribute, or bypass those foreign
  blobs.

## 2026-08-24 r9 physical-split confirmation

- On current HEAD `f811b3bf474d70347199772a175422333dfb36f6`, fresh read-only
  `cargo tree -p zr_rhi --edges normal,build,dev --locked` exits 0 with 23 output lines and zero
  `wgpu` or `zr_rhi_wgpu` matches. The neutral crate therefore still has no normal, build, or dev
  reverse dependency on its backend.
- Fresh production-domain audit reports 2,789 references / 71 domain edges and no
  `rhi -> rhi_wgpu` edge. These two results reconfirm the physical dependency direction; they do not
  exercise the backend or Runtime product.
- Managed Rust gates were not repeated because the four routed Runtime90 diagnostics blobs retain
  the exact compile-failing hashes recorded by the 2026-08-24 handoff. Focused backend, Runtime,
  Editor, immutable review, fixed return, and service commit remain pending, so this Failure stays
  `open` and M2 remains unaccepted.

## 2026-08-24 Runtime90 surface expansion routing

- Windows managed editor production job `b8c230e7d5da41bd855c1b2d3fa82278` reached the current
  physical `zr_rhi` crate and stopped after about 29.6 seconds with two E0499 diagnostics at
  `zr_rhi/src/surface.rs:233-234`. Both diagnostics come from returning simultaneous mutable
  references to the counter and active-set fields through a `match` over the mutex-guarded state;
  no Runtime/Editor/IBL source was compiled after that dependency failure.
- This is not a regression in the completed neutral/backend physical split. The current
  `surface.rs` is an unowned added Runtime90 product-expansion blob with SHA-256
  `eb7195b710bbd6d9748d4151cbbc52e8039807316f24b0b17c3b7e7dd1ce1d50`; the active Runtime90 r3
  immutable scope covers capability/device owners but does not include this file. Frameworks01
  plan authorization request `e83ad5bf06ad4e67858f87b4d241f1a6` for a Runtime90 child failure
  was rejected as `outside_registered_child`, so Frameworks01 did not reattribute or edit the
  Runtime90 surface implementation.
- Reference review preserves the intended dependency and lifecycle direction. Unreal keeps
  `FRHIViewport` as an RHI resource and routes create/resize/end-drawing through `FDynamicRHI`;
  Bevy stores backend `SurfaceData`, acquires a `SurfaceTexture`, and consumes it on present;
  Godot's D3D12 owner releases swapchain buffers before resize and rejects a zero-sized surface.
  These references support a backend/device-owned surface lifecycle with opaque neutral receipts.
  They do not justify moving WGPU construction into `zr_rhi`, exposing backend objects to Editor,
  or adding a compatibility path to bypass the allocator.
- The fixing Runtime90 successor must first own the exact surface blob, add focused session/frame
  allocation and stale-release coverage, then update the two counter/set pairs without borrowing
  disjoint fields through a returned tuple. Upward verification remains the focused `zr_rhi` gate
  followed by the original editor production build. Until that occurs, this Frameworks01 Failure
  remains `open`; no compile GREEN, fixed return, milestone commit, or performance claim is made.

## 2026-08-25 Runtime90 surface current-source re-audit

- Current HEAD is `0fd7df4ecdd157f9505cd51013780e3225cfb83c` at coordinator baseline epoch
  435. `zircon_runtime/crates/zr_rhi/src/surface.rs` has drifted to SHA-256
  `4341390f9d6991210a2c9bb1e2dbf8a20c7b4fb01cc51b33157a095b77af47c6`; ownership request
  `cbc68bfaf7f24a8b9485378c2297b8ac` still reports `attribution_missing`, no owner, and no live
  lease. The former Runtime90 r3 primary is now cancelled. Its active r4 successor
  `rhi90-native-lifecycle-repair-r4-1b2684b4-20260825` is `resolving_failure`, but its immutable
  native-lifecycle scope still does not contain this file; Plan 90 WIP therefore cannot be bypassed
  by registering a second primary.
- The current `allocate()` no longer returns simultaneous mutable references. It first reads the
  selected counter, computes the checked successor, then updates the counter and active set in one
  match arm. This is a plausible forward repair for the two E0499 diagnostics, but it remains an
  unowned current-source change and has no managed compile receipt; the old editor build cannot be
  reclassified as passing.
- Existing module tests cover session/frame release-to-stale and bounded terminal history. They do
  not explicitly lock monotonic allocation, foreign allocator rejection, device-generation
  rejection, or counter/namespace exhaustion. More importantly, surface handles use two monotonic
  `u64` counters plus `HashSet` membership, while the same crate's resource handles already use a
  generational slot table, freelist reuse, and generation-overflow retirement. A Runtime90 owner
  must profile the expected surface/frame churn and choose one coherent device-handle policy before
  claiming algorithmic or memory optimality; Frameworks01 will not turn the borrow-check repair into
  an unmeasured structural rewrite.
- Unreal's current RHI keeps viewport creation, resize, and end-drawing on the `FDynamicRHI` owner,
  with `FRHIViewport` as an RHI resource. This continues to support backend/device-owned lifecycle
  state, not an Editor-owned allocator or a compatibility facade. The required closure remains:
  exact-blob Runtime90 ownership, focused contract/profile evidence, managed `zr_rhi` tests, then the
  original Runtime/Editor product gate. The Failure stays `open`.

## 2026-08-28 WGPU surface lease fail-closed repair and algorithm re-audit

- Current shared HEAD is `a2d8d811c4a3a1fc1db6f5375c491e7e4502533f` at coordinator baseline epoch
  539. The Runtime90 owner of the current WGPU surface blobs is archived; ownership matrix reported
  `device/surfaces.rs`, `production/surface.rs`, `production/device/surface_lifecycle.rs` and
  `device.rs` unowned with no live lease. Frameworks01 claimed and attributed those exact paths
  because they already belong to this Session's immutable WGPU backend scope. The foreign
  `zr_rhi/src/surface.rs` allocator remains outside scope and was not edited.
- Whole-module review found a concrete correctness violation after the earlier product surface
  expansion. `SurfaceFrameLease::new` is public, but deterministic and production present/discard
  consumed only `frame.frame()`. A forged lease could therefore reuse a live frame id with a foreign
  session, target, default view, or descriptor and terminalize the real acquired frame. Production
  discard also enumerated and cancelled that real frame's Accepted submissions before any complete
  lease validation.
- The forward repair makes both backend owners retain and compare the complete lease identity.
  Deterministic present validates before submission-to-target lookup and discard validates before
  terminalization. Production present consumes `&SurfaceFrameLease`; discard validates before
  ticket cancellation and again at the terminal owner. Trusted session/device teardown remains on
  a private frame-id path so it cannot be forged through the public contract. A deterministic
  behavior regression creates two live frames, forges all four non-frame identity components, and
  requires both real leases to remain independently discardable after every rejection.
- The implementation does not add a compatibility constructor, facade, alias, second surface owner,
  queue submit, device wait, or global lock. Scoped `rustfmt --check` and `git diff --check` are
  GREEN; the complete-identity/order source invariant is 8/8 GREEN. Current file sizes are
  `device.rs 769`, `device/surfaces.rs 483`, `production/surface.rs 586`, and
  `production/device/surface_lifecycle.rs 163`, all below the 800-line soft budget. The global docs
  convention gate remains shared-tree RED at 1,514 violations across 401 documents, while this
  Failure and the new owned preflight record have 0 violations.
- Fresh production dependency audit completes on current source with 3,258 references / 72 domain
  edges and no `rhi -> rhi_wgpu` edge. Unreal's `FDynamicRHI`/`FRHIViewport` and D3D12 viewport owner
  still place create/resize/end-present and backbuffer teardown in the RHI lane. Bevy keeps one
  consuming `Option<SurfaceTexture>` per extracted window and forbids configure while an old
  acquired texture survives. These references support complete backend-owned lease consumption;
  they do not justify an Editor owner or a second native surface path.
- The same review found that acquire/session teardown still scan every active frame and the
  deterministic submission mirror scans every `frame * command` pair. This contradicts the older
  source claim that acquire is average `O(1)`, but there is no timing evidence that it dominates the
  current single-window product. The required pre-profile, workload matrix, WPR/WPA/power gates and
  possible session-local active-frame/reverse-target index are recorded in
  [2026-08-28-rhi-wgpu-surface-session-index-preflight.md](2026-08-28-rhi-wgpu-surface-session-index-preflight.md).
  Index optimization remains `not_started` until managed profile data confirms the bottleneck.
- Managed `zr_rhi_wgpu` behavior/compile tests, the wider Runtime/Editor gates, real native window,
  PNG/RenderDoc, WPR power data, immutable review, fixed return and service commit remain pending.
  The earlier Cargo acquire has only an accepted receipt and must not be polled or duplicated.
  Therefore this Failure remains `open`; Frameworks01 M2 and the enclosing goal are not accepted.
