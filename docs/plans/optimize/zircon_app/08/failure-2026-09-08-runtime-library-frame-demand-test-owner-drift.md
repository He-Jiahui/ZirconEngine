---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: runtime-library-frame-demand-test-owner-drift
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
fixing_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/runtime/10
fixing_child_dir: docs/plans/optimize/zircon_app/08
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/frame_demand.rs
tests:
  - cargo test -p zircon_app --no-default-features --features diagnostic-log --locked --lib entry::runtime_library::tests::
---

# App08: frame-demand source assertion checks the wrong owner

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 来源执行切片：V8 ABI inventory's direct App loader acceptance.
- 修复责任计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 交接原因：The assertion and production frame-demand consumer belong to App08.
- Related origin: [Runtime10 V8 inventory](../../../zircon_runtime/runtime/10/failure-2026-09-08-runtime-v8-abi-inventory-test-drift.md).

## 失败现象与复现证据

Windows managed job `3204ef6e6c1b46ffaee78707093ed4dd` ran the declared command:
26 passed, 2 failed, 1 ignored. Frozen input
`runtime10-v8-abi-owner-3188-20260908` has manifest
`ffd0d523c54a6e8732e2f1613bfedee4276a07b535736a198362a4aa264d412a`.
Its `results/runtime10-v8-app-loader-diagnostics-3188.{json,log}` retains evidence.
`runtime_library_hard_cuts_to_v7_allocation_contract` fails at tests.rs:359 because
`frame_demand.contains("RuntimeFrameDemand::try_from")` is false.
The separate [junction fixture failure](failure-2026-09-08-runtime-library-junction-fixture-creation.md)
owns the other failure. The ignored project capture test requires excluded zr_vm.

## 最低共享层根因

The conversion call resides in `RuntimeSession::tick_frame` in runtime_session.rs.
The child frame_demand.rs owns `impl TryFrom<ZrRuntimeFrameDemandV1>` and the checked
conversion implementation, not that caller. The static owner assertion checks the
implementation file for a caller expression. The dynamic checked-conversion test
already passed in this same job; the failure does not establish a conversion bug.

## 架构修复验收

- Check the actual caller and conversion owner without duplicating production
  code or reintroducing retired table versions.
- Preserve dynamic unknown-kind, malformed-delay and maximum-delay checks.
- Execute the original regression and App loader batch, resolve the independent
  junction failure, then complete managed validation and C0/I0/M0 review.

## 禁止临时方案

- Do not add a matching string or duplicate conversion to production just to
  satisfy a source assertion. Do not enable zr_vm or count its ignored test as a pass.

## 修复结果与回传

State: `active-owner-repair_validation-blocked`.
The App08-owned source repair now checks the actual caller in
`runtime_session.rs`; `tests.rs` SHA-256 is
`61b5cfc109c543ebe12e6681d2ac496ed816e963d4702c27fb4549d8b1c26046`.
The Windows fixture uses `ProjectPaths::display_path`, `cmd.exe`, and
`CommandExt::raw_arg` so shell quoting is separate from filesystem paths.
The repair is attributed to `build-reuse-app08-20260905`; no other source is
claimed.

Managed job `4fc81a2635d945cf8acce0a0d1df369d` ran the locked static loader
filter against source manifest `8cee2b4b1143fd8d0117804e832742c038dc12b6d78822e17825548528b425d3`.
It stopped during `cargo check` with exit 101 on the existing Runtime
`native_artifact_trust.rs` `E0716` borrow error, so the App tests did not run.
The receipt records 354 verified dependency packages, queue/sync/check of
28.74/728.17/167.48 seconds, and zero compile/link or test seconds. The
original regression and upward loader batch therefore remain unaccepted;
failure return and independent review are pending the Runtime owner clearing
that compile blocker.
