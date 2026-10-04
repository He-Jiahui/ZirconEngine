---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-13
summary_slug: validation-copy-workspace-service-import
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
resolved_at: 2026-09-18
---

# validation-copy-workspace-service-import: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 full validation-copies regression replay
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator01 full validation-copies regression replay` — python -B -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_external_git_source_uses_pinned_commit_and_survives_restart -v

## 最低共享层根因

The regression test constructs WorkspaceCopyService directly after restart but does not import that symbol; the helper-local import is not visible in the test method scope.

## 架构修复验收

- The exact restart regression executes and passes without NameError.
- The complete validation-copies source and workspace-copy regression suites execute with zero errors.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：ValidationCopySourceTests restart regression constructed WorkspaceCopyService directly without a method-visible import; the helper-local import was not visible. The initial narrow managed ticket also omitted the audited 38-path coordinator import closure, so it was not reused as acceptance evidence.
- 架构修复：Added the direct WorkspaceCopyService import at the restart regression call site and sealed the complete audited import closure in the managed validation ticket; no alias, shim, fallback, or test bypass was added.
- 验证：Managed Windows Python ticket 3331e178a3134441bb9b7c8d86d89b19, copy 10ca14b5f7684f8890b92bb99cdcc7ad, run 3331e178a3134441bb9b7c8d86d89b19 executed python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v from the immutable 38-path source manifest 200895f67ddafd3e3ff3a6a9fb2d8ca8334068d66a26e7b4e95e8c4b78d2eed4; exit 0, 150 tests passed, stderr ended Ran 150 tests in 863.862s OK.
- 回传：Coordinator generated the canonical fixed and return artifacts for validation-copy-workspace-service-import; managed ticket, immutable copy, run, source manifest, and 150-test result are bound to this return.
