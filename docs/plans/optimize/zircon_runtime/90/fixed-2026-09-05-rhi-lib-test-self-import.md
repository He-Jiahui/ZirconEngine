---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-28
summary_slug: rhi-lib-test-self-import
origin_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/90
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi/src/tests/boundary.rs
  - zircon_runtime/crates/zr_rhi/src/tests/device_fault.rs
  - zircon_runtime/crates/zr_rhi/src/tests/device_profile.rs
  - zircon_runtime/crates/zr_rhi/src/tests/handles.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -TestFilter surface_handle -VerboseOutput
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -VerboseOutput
resolved_at: 2026-09-05
---

# Runtime90: RHI unit tests import their own crate as an external dependency

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 来源执行切片：Runtime90 RHI library test validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 交接原因：失败位于 Runtime90 所有的 neutral RHI unit-test tree；无上层替代路径。

## 失败现象与复现证据

Runtime90's managed `zr_rhi` surface-handle gate compiled the library test target on
2026-08-28 and failed before executing a test. Rust reported E0432 in `boundary.rs`,
`device_fault.rs`, `device_profile.rs`, and `handles.rs`: each unit-test child used
`use zr_rhi::...`, but a crate's `src/tests` modules must consume the current crate
through `crate::...`.

## 最低共享层根因

The four tests were written with integration-test import syntax even though `lib.rs`
mounts them under `#[cfg(test)] mod tests`. Adding `extern crate self as zr_rhi` would
hide that ownership error behind a permanent alias; the correct boundary is direct
crate-relative imports in the unit-test tree.

## 架构修复验收

- Replace only the four invalid self-imports with `crate::...`; retain every test body.
- Do not add a self alias, compatibility facade, feature bypass, or integration-test copy.
- The managed surface-handle filter must compile the complete lib-test target and run its
  selected tests successfully.
- The full managed `zr_rhi --lib` suite must pass before this record returns fixed.

## 禁止临时方案

- 不添加 self alias、兼容 facade、feature bypass、integration-test copy 或测试削弱。

## 修复结果与回传

- 根因：Unit-test modules used external-crate imports even though lib.rs mounts them as cfg(test) children; the RHI UI source-contract tests also had stale source slicing that masked the current generic HashMap and test-module boundaries.
- 架构修复：Kept unit tests crate-relative and corrected the two source-contract assertions to inspect the explicit typed capacity allocation and production section before the test module. No alias, compatibility facade, fallback, or production behavior bypass was added.
- 验证：Coordinator-managed Windows cargo test -p zr_rhi --locked --lib completed successfully in the E:\cargo-targets\zircon-engine\pool\512c70185e168c08f5932544e6456e4327df2ef2717a85aa184d3337f9b3c6eb target: 87 tests, 83 passed, 2 ignored, 0 failed; scoped Python regressions 2/2 passed; rustfmt check passed.
- 回传：Runtime90 self-import failure is fixed. The complete managed zr_rhi library gate is green; remaining Runtime90 failures and product-level acceptance remain open.
