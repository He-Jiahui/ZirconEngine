---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-13
summary_slug: validation-copy-manifest-missing-cargo-target-selection
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
resolved_at: 2026-09-18
---

# validation-copy-manifest-missing-cargo-target-selection: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 managed immutable-copy validation replay
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator01 managed immutable-copy validation replay` — python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v (managed ticket eecf311ccf834745b979b0714486f7d4)

## 最低共享层根因

The sealed source manifest omitted tools/session_coordinator/cargo_target_selection.py, which validation_copies.py imports; the immutable copy therefore cannot import either regression module although the workspace source passes locally.

## 架构修复验收

- The managed immutable copy contains cargo_target_selection.py and imports both requested regression modules.
- The exact validation-copies and workspace-copy suites execute with zero errors in the immutable copy.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：The sealed immutable source manifest omitted tools/session_coordinator/cargo_target_selection.py, which validation_copies.py imports, so the copy failed before the requested regression modules could collect.
- 架构修复：The coordinator now seals the complete reviewed import closure, including cargo_target_selection.py, in the immutable copy; no alias, shim, fallback, or test bypass was added.
- 验证：Managed Windows Python 3.14 ticket 3331e178a3134441bb9b7c8d86d89b19, copy/job 10ca14b5f7684f8890b92bb99cdcc7ad, run 3331e178a3134441bb9b7c8d86d89b19, source manifest 200895f67ddafd3e3ff3a6a9fb2d8ca8334068d66a26e7b4e95e8c4b78d2eed4; python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v; 150 tests ran and OK.
- 回传：Coordinator generated the canonical fixed-* and return artifacts and atomically removed the original child failure record after the complete immutable-copy replay.

## 2026-09-13 rolling repair evidence

The fixing Session sealed the missing `cargo_target_selection.py` path together
with the complete changed coordinator import closure. Managed immutable Windows
ticket `eba16be624154fe4955188bdf6d0858f` used source-manifest hash
`738b841e773cfd1ef2d3dc5fc8680886f7e41b0f3be7484838b5ee2713cbba3f` and
executed `python -B -m unittest tools.session_coordinator.tests.test_validation_copies -v`
in copy/job `9953d3c2c547447692ca792c1a59aceb` (run with the same ticket ID).
The immutable Windows Python 3.14 run completed with exit code 0 and `33/33`
tests passed, including the compile-time resource and pinned-baseline closure
regressions that previously failed before collection.

This proves the validation-copies half of the requested replay. The exact
workspace-copy suite still cannot be sealed from this Session's `a8e2f6aa...`
base because its worker imports reviewer-owned `validation_ticket_policy.py`;
that dependency is recorded in the separate open
`validation-copy-manifest-missing-validation-ticket-policy` failure. Therefore
this artifact remains open until the workspace-copy half executes from a
legitimate reviewed immutable manifest.
