---
handoff_kind: fixed
status: fixed
failure_scope: cross_plan
created_at: 2026-08-23
summary_slug: runtime90-readback-layout-test-queue-lifetime
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/tests.rs
resolved_at: 2026-09-05
---

# Runtime90 readback layout test Queue lifetime: validation failure handoff

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 full lower-layer WGPU regression validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 交接原因：Runtime90 explicitly owns the WGPU readback queue and
  `gpu_readback_queue/tests.rs`.

## 失败现象与复现证据

- The Windows managed `zr_rhi_wgpu --lib` run compiles successfully but exits 101 with 220/221
  tests passing. The sole failure is
  `gpu_readback_queue::tests::readback_layout_failure_preserves_callbacks_for_abort`.
- A focused replay of the managed test executable reproduces the failure at 0/1. The backtrace
  stops in `wgpu_core::device::resource::Device::create_command_encoder`, before
  `GpuReadbackQueue::encode_copies` or its expected `CapacityOverflow` result is reached.
- The test binds `let Some((device, _)) = offscreen_test_device()`, which drops the sole returned
  `wgpu::Queue` immediately, and then calls `device.create_command_encoder`. The other readback
  tests that encode copies retain the returned queue as `submission_queue` until after command
  submission.
- The UI12-owned `ui_surface::tests::native_submission` matrix remains green at 14/14 in the same
  current-source test binary, including both rounded-coverage regressions.

## 最低共享层根因

The test is intended to exercise staging-layout overflow and callback retention, but its fixture
releases the WGPU Queue before creating a command encoder. With wgpu 29, command-encoder creation
then dereferences a missing queue-side device resource inside wgpu-core. The test therefore fails
in fixture lifetime setup instead of reaching the readback contract under test.

## 架构修复验收

- Retain the queue returned by `offscreen_test_device()` for the complete test lifetime; an
  intentionally unused named binding such as `_submission_queue` is sufficient.
- Preserve the existing oversized readback range, `CapacityOverflow` assertion, pending callback
  assertion, explicit abort, and exactly-once abort callback assertion.
- Re-run the focused test and the complete managed `zr_rhi_wgpu --lib` suite.

## 禁止临时方案

- Do not ignore the test, catch the wgpu panic, or weaken the expected `CapacityOverflow` result.
- Do not alter production readback capacity logic to accommodate a test fixture that has already
  released its Queue.

## 修复结果与回传

- 根因：The test discarded its only wgpu Queue before creating the command encoder, so fixture setup failed before exercising readback overflow.
- 架构修复：The existing named _submission_queue binding retains the Queue for the full fixture lifetime. Oversized range, CapacityOverflow, pending callback and explicit abort assertions remain unchanged; production readback logic is unchanged.
- 验证：Focused replay of the current managed test binary passed 1/1 on 2026-09-05 using fixture lease f316d891bf164d6dbe972e5c8cb32347. Full serial Windows validate-matrix.ps1 -Package zr_rhi_wgpu -SkipBuild -LibTests with RUST_TEST_THREADS=1 passed: job 045109d2b7624a78ad329f307f4416b9 released exit 0 at 2026-09-05T08:44:16Z, [OK] Cargo test. No numeric full-suite total is claimed because the validator suppresses harness stdout.
- 回传：UI12 can resume the M6 full lower-layer WGPU regression gate; queue lifetime setup now reaches the intended readback overflow and abort contract.
