---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: runtime-library-junction-fixture-creation
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
fixing_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/runtime/10
fixing_child_dir: docs/plans/optimize/zircon_app/08
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
tests:
  - cargo test -p zircon_app --no-default-features --features diagnostic-log --locked --lib entry::runtime_library::tests::runtime_library_default_path_uses_the_physical_product_directory_identity
---

# App08: product-directory junction fixture fails before path validation

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 来源执行切片：V8 ABI inventory's direct App loader acceptance.
- 修复责任计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 交接原因：App's loader fixture owns product-directory alias construction.
- Related origin: [Runtime10 V8 inventory](../../../zircon_runtime/runtime/10/failure-2026-09-08-runtime-v8-abi-inventory-test-drift.md).

## 失败现象与复现证据

Windows managed job `3204ef6e6c1b46ffaee78707093ed4dd` ran the full loader filter
with diagnostic-log, no default features and locked Cargo: 26 passed, 2 failed,
1 ignored. Input `runtime10-v8-abi-owner-3188-20260908`, manifest
`ffd0d523c54a6e8732e2f1613bfedee4276a07b535736a198362a4aa264d412a`;
logs `results/runtime10-v8-app-loader-diagnostics-3188.{json,log}`.
`runtime_library_default_path_uses_the_physical_product_directory_identity`
panics at tests.rs:631. The mklink subprocess reports
`Local volumes are required to complete the operation.`

## 最低共享层根因

The proven boundary is `create_directory_link`: the fixture formats paths into
`cmd /D /S /C mklink /J` and fails before invoking the production path resolver.
The actual temporary path representation and mklink acceptance still require
diagnosis. This evidence does not prove that production path canonicalization is
wrong, and no unobserved prefix or storage assumption is asserted as the root cause.

## 架构修复验收

- Record the actual managed temporary path and reproduce junction construction
  on the assigned Windows local volume; repair the fixture at its shared owner.
- Execute the original test through the physical-product identity assertion.
- Re-run the App loader batch after the independent
  [frame-demand test repair](failure-2026-09-08-runtime-library-frame-demand-test-owner-drift.md).
- Complete formal validation and C0/I0/M0 review before return and closeout.

## 禁止临时方案

- Do not skip the junction test, turn creation failure into a pass, or replace
  physical identity with lexical equality. Do not redirect managed output to an
  unapproved drive, enable zr_vm, or edit the active owner's source without transfer.

## 修复结果与回传

State: `active-owner-repair_validation-blocked`.
The App08-owned fixture repair is attributed to
`build-reuse-app08-20260905`; the current `tests.rs` SHA-256 is
`61b5cfc109c543ebe12e6681d2ac496ed816e963d4702c27fb4549d8b1c26046`.
It passes display-form paths to a raw `cmd.exe /D /S /C` command and preserves
the physical-directory assertion; no lexical fallback or shared Runtime path
change was added.

Managed job `4fc81a2635d945cf8acce0a0d1df369d` reached the prerequisite
`cargo check` but stopped with the existing Runtime
`native_artifact_trust.rs` `E0716` borrow error. Consequently the original
junction reproduction and the combined App loader batch have not executed,
and failure return plus independent review remain pending the Runtime owner
clearing that compile blocker.

Runtime04 subsequently isolated both tool-boundary defects in its independently
owned junction fixture: operational verbatim paths are rejected by mklink,
and passing the quoted shell program via `Command::arg` adds incompatible
C-runtime escaping. Its path-view-only candidate still failed; the complete
display-path plus `CommandExt::raw_arg` correction passed all 10 package-assets
tests as managed job `836e5a3c4dd347929364305b085984a6`. Exact source, RED,
path-only failure and GREEN receipts are linked from
[Runtime04's canonical junction failure](../../../zircon_runtime/runtime/04/failure-2026-09-08-project-root-junction-verbatim-fixture.md).

This supplies a tested adjacent-owner repair pattern, not App acceptance.
App's original helper and hash above remain unchanged under
`build-reuse-app08-20260905`; its actual operands and loader identity assertions
still require App-owned validation. No Runtime dependency or shared production
path API should be added merely to reuse a test-fixture implementation.
