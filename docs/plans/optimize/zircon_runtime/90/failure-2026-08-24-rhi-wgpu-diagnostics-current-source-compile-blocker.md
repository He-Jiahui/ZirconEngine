---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: rhi-wgpu-diagnostics-current-source-compile-blocker
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/diagnostics.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/layout.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/query.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/resource_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/completion_order.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/service.rs
tests:
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked
  - cargo build -p zircon_runtime --locked
---

# Runtime90: repair current-source WGPU diagnostics compile closure

Canonical fixing-plan artifact; Frameworks01 remains the originating product-build gate.

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M1 `zr_math` physical hard cut product build
- 修复责任计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 交接原因：Frameworks01 的新 math/Interface 层已经通过 focused gates，并在 Runtime build 中
  编译完成；产品编译随后停止在 4 个不属于 Frameworks01 的 WGPU diagnostics/resource
  validation blobs。

## 失败现象与复现证据

Managed job `246fdaf5d6c443f9b71149d744b5675e` ran locked
`cargo build -p zircon_runtime` on the coordinator-owned `D:` pool. It successfully compiled
`zr_math` and `zircon_runtime_interface`, then failed after 83.52 seconds with five current-source
errors in `zr_rhi_wgpu`. This proves the Frameworks01 M1 math layer reached the Runtime product
boundary; it does not establish a Runtime GREEN gate.

Exact current hashes at coordinator baseline epoch 418 are unowned and report
`attribution_missing`:

- `production/diagnostics/query.rs`:
  `ab36a7c7d011f04656e79df298fe80f2bfdf40f79d0ab0505bae1949047e64cb`;
- `resource_validation.rs`:
  `8d3c1ddc22a286b485238819d83c4b5905fd885a29a993479121f133f88ef469`;
- `production/diagnostics/readback/completion_order.rs`:
  `87c5ce42853d0a7e0654220d9052d4b474635408991cfea73d06e5260a5c325c`;
- `production/diagnostics/readback/service.rs`:
  `7e76bb02b994ef0e2a926fc20e1297d1671ce92b35f8d90d6db79f8a820a562a`.

The compile diagnostics are:

- E0425: missing local `tracker` in `production/diagnostics/query.rs:430`;
- E0433 and E0609: missing `BindingResourceType` import plus stale `entry.resource` field in
  `resource_validation.rs:196`; the current descriptor field is `resource_type`;
- two E0277 errors: derived `Default` on
  `TicketOrderedDiagnosticCompletions<DiagnosticBatchCompletion>` imposes an unsatisfied
  `DiagnosticBatchCompletion: Default` bound at query/readback service construction sites.

## 最低共享层根因

The diagnostics/readback implementation is an in-flight Runtime90 current-source owner, not a math
or Runtime facade concern. The active Runtime90 primary session
`rhi90-operation-capability-contract-m0-r3-bee4c707-20260822` owns the plan family but its immutable
scope does not contain these four exact blobs. The files therefore have neither a live lease nor an
attributed implementation owner, and Frameworks01 cannot repair or absorb them legally.

## 架构修复验收

- Runtime90 rotates scope or registers a dedicated RHI diagnostics session for all four exact
  current hashes before editing.
- Restore a single coherent diagnostics tracker lifecycle; do not hide the missing local with a
  dummy tracker or skip terminalization.
- Validate bind-group descriptors against `resource_type` using the neutral `zr_rhi` contract;
  do not recreate a backend-only shadow descriptor field.
- Implement `Default` for `TicketOrderedDiagnosticCompletions<T>` without requiring `T: Default`,
  or construct its empty state explicitly, while preserving ordered completion semantics.
- Run focused `zr_rhi_wgpu` diagnostics/readback/resource-validation tests, then locked
  `cargo build -p zircon_runtime` on a coherent coordinator snapshot.
- Return the fixed record with exact hashes and managed job evidence before Frameworks01 reruns
  Runtime/App/Editor acceptance.

## 禁止临时方案

- Do not change `zr_math`, Runtime Interface, or Runtime math facades to bypass WGPU compilation.
- Do not disable diagnostics/readback modules, remove validation, or add compatibility fields.
- Do not attribute the current unowned blobs to Frameworks01 or fold them into the math candidate.

## 修复结果与回传

Open. Frameworks01 left all four RHI blobs unchanged. Runtime/App/Editor product acceptance remains
blocked until Runtime90 returns a validated current-source fix.

## 2026-08-24 epoch-420 re-audit

- Current HEAD is `f811b3bf474d70347199772a175422333dfb36f6`. All four blocker hashes remain
  byte-for-byte identical to the epoch-418 evidence above: `ab36a7c7...`, `8d3c1ddc...`,
  `87c5ce42...`, and `7e76bb02...`.
- Active Runtime90 session `rhi90-operation-capability-contract-m0-r3-bee4c707-20260822` remains
  `resolving_failure`, but its immutable scope still does not contain these query,
  resource-validation, completion-order, or service blobs. Frameworks01 therefore cannot edit or
  attribute them.
- Because neither source nor ownership changed, no duplicate Runtime Cargo job was launched. The
  original managed compiler diagnostics remain the current reproduction evidence, and all upstream
  Runtime/App/Editor product gates remain pending.

## 2026-08-27 Editor product-build re-audit

- Managed current-source Job `582144d2c24f4d0684533088bffe4f69` built from HEAD
  `ea35974cdf64068f6789010451d20bbf69e0a29d` plus the shared worktree and reached
  `cargo build -p zircon_app --bin zircon_editor --no-default-features --features
  target-editor-host --locked`. It finished with exit code 101 and was released; its ephemeral
  target was deleted by the coordinator.
- The previous diagnostics in this artifact no longer appear. The complete current compiler
  blocker set is two E0308 errors in the active Runtime90 diagnostic-readback cutover:
  `production/device/diagnostics.rs:627` and `:684` pass `u32 copy_row_bytes` to
  `DiagnosticTextureReadbackLayout::new`, whose owner in
  `production/diagnostics/readback/layout.rs:14` now requires `u64` so row alignment and staging
  byte-length arithmetic remain overflow checked in the wider domain.
- Current untracked-source SHA-256 values are
  `C032E024292482E4A6DBF2ED0F97900E9B58E821E801B8727B6A56CDB12F1436` for
  `production/device/diagnostics.rs` and
  `9B417EB768FA39593B9F523EDA265517752809642F06236C7AB9530E70780C15` for
  `production/diagnostics/readback/layout.rs`. Both files belong to the in-flight Runtime90
  product diagnostic-readback owner cutover and remain untouched by UI12.
- The architectural repair is a checked-domain-preserving widening at both call boundaries
  (`u64::from(copy_row_bytes)` or an equivalent source-owned calculation), not a narrowing of the
  layout constructor back to `u32`. Runtime90 must run its focused layout/readback checks first,
  then rerun the locked Editor product build before UI12 can resume product WGPU capture.
