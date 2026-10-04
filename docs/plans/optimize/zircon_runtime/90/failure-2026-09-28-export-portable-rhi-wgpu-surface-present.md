---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: export-portable-rhi-wgpu-surface-present
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi/src/native_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/surface_lifecycle.rs
tests:
  - cargo +1.94.1 test -p zr_rhi_wgpu --lib --locked tests::surface_lifecycle:: -- --test-threads=1
  - cargo +1.94.1 test -p zr_rhi_wgpu --lib --locked production::tests::surface_bootstrap -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked graphics::tests::surface_targets -- --test-threads=1
---

# Runtime90: portable RHI/WGPU export surface and present

## 来源执行者

- 来源计划：docs/plans/zircon_plugins/09-export-publishing.md
- 来源执行切片：Plugins09 WOC mobile/browser generated host real-surface and presented-frame acceptance
- 修复责任计划：docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
- 交接原因：The lowest production RHI/WGPU backend has no Android, iOS or browser native target creation or presentation path.

- Origin: Plugins09 generated mobile/browser host acceptance, recorded in `docs/plans/zircon_plugins/09/failure-2026-07-17-woc-mobile-browser-host-noop.md`.
- Fixing owner: Runtime90 production RHI/WGPU native surface target, acquisition, configuration, present and teardown. Runtime57 owns the separate App/ABI host target and lease handoff in `docs/plans/optimize/zircon_runtime/57/failure-2026-09-28-export-portable-surface-target-host-lifecycle.md`; Runtime191 supplies current working tree architecture evidence.
- The open Runtime90 `rhi-surface-allocator-borrow-compile` lifecycle is a distinct allocator/borrow defect with its own waiting-validation Session; it does not cover portable targets. This child record carries the open link without editing the plan definition.

## 失败现象与复现证据

`RenderNativeSurfaceTarget` currently has only Win32. Production WGPU `create_native_surface` constructs only that target and returns `SurfaceUnavailable` for it on non-Windows builds. Thus a generated Android, iOS or browser host cannot acquire a production game-frame surface, let alone present a frame, even after the App/ABI boundary admits a host target.

At the read-only 2026-09-28 boundary, exact SHA-256 values were: `zr_rhi/src/native_surface.rs` `de43e4ce25c16b9c7b863d3c568564a1f434057d4a95c5a6f68d3fcc984995e5`; WGPU `production/surface.rs` `54ad83fbb5fd206d807f6f55489d6d9308029e71449f024a59e1ff45457fb2c4`; `production/surface_bootstrap.rs` `415f695565f9eaca2dbaeeed8bdf2c5459ca349cded27345f70368bc113e3e9c`; `production/device/surface_lifecycle.rs` `b5676e73dba1eb2c34c0752fc21cee4208aebf33cecf3301e51c2913cd442934`. These are source observations, not platform runs.

The existing frontmatter tests exercise deterministic session/frame lifecycle, source-level bootstrap wiring and runtime surface routing. No current test acquires and presents Android, iOS or browser native targets. The Plugins09 live mobile/browser filters are also absent today.

## 最低共享层根因

RenderNativeSurfaceTarget and WGPU production surface creation are Win32-only. App/ABI target admission belongs to Runtime57; backend acquisition, present and teardown belong here.

## 架构修复验收

- Implement typed production surface targets and WGPU acquisition for Android native window, iOS view/Metal layer and browser canvas where supported. Keep the native target and device/session leases alive until acquired frames are presented or discarded and the surface is destroyed.
- Add target-specific tests on real Android, iOS and browser/WASM runners: acquire, submit and present at least two nonblank project frames; reconfigure after physical-size/DPI/orientation or canvas-size changes; reject old target generations; settle all frames and release before pause, context loss, unload or native target destruction; resume/rebind and present again where supported.
- Run lower RHI/WGPU lifecycle tests and the original runtime routing reproduction, then exact new platform tests with nonzero selected counts. Once Runtime57's target handoff is available, rerun the Plugins09 generated-host tests and actual exported products. If the shared contracts must land before these product gates, use scoped integration with `integrated_validation_pending` until full acceptance.
- Keep retained-UI `ui_surface/surface_setup.rs` outside this game-frame repair unless the exported runtime UI surface is proven part of the product gate. Any such additional consumer must receive matching tests.

## 禁止临时方案

Do not fake a Win32 target, acknowledge frames without present, silently fall back to a headless texture, or merge this backend defect into the allocator/borrow failure. This failure remains open; no Cargo, product run, failure return, review or closeout pass is claimed.

## 修复结果与回传

Open state: pending repair and managed acceptance; no pass is claimed.
