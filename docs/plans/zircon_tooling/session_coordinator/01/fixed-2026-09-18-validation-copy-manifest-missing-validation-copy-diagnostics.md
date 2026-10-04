---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-14
summary_slug: validation-copy-manifest-missing-validation-copy-diagnostics
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
resolved_at: 2026-09-18
---

# validation-copy-manifest-missing-validation-copy-diagnostics: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 managed immutable-copy validation replay after the cargo-target-selection manifest closure repair
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator01 managed immutable-copy validation replay after the cargo-target-selection manifest closure repair` — python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v (managed ticket 8624c7d5eb50486cbdbe401da13088c7)

## 最低共享层根因

The sealed source manifest still omits tools/session_coordinator/validation_copy_diagnostics.py, which workspace_copy.py imports; the immutable copy therefore raises ModuleNotFoundError before the requested regression suite can execute.

## 架构修复验收

- The managed immutable copy contains validation_copy_diagnostics.py together with every imported coordinator module.
- The exact validation-copies and workspace-copy suites execute with zero errors in the immutable copy.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 当前证据

- 受管票据 `8624c7d5eb50486cbdbe401da13088c7` 在不可变副本中真实执行了 34 项，
  其中 7 项因 `ModuleNotFoundError: tools.session_coordinator.validation_copy_diagnostics`
  在导入阶段失败；该票据已保留为失败证据，未复用为通过。
- 使用同一基线提交和依赖根的临时不可变副本，补入完整的 14 个新增导入模块后，
  `python -B -m unittest tools.session_coordinator.tests.test_validation_copies
  tools.session_coordinator.tests.test_workspace_copy -q` 实际执行 `150` 项并以
  `OK` 结束。该结果是本地闭包探针，不替代新的受管票据。
- 下一张受管清单还需保留当前 owner 的新增模块，并在协调器恢复后重新密封；
  `validation_preflight.py`、`validation_ticket_policy.py` 和
  `validation_timings.py` 仍由独立审查 Session 持有，未越权转移。

### 受管下层回归（2026-09-13）

- 票据 `3737473ea09a49e99efede53a05487f9` 使用基线
  `a8e2f6aa3e42f832500a770f9f48ce188e6d9773`、受管 Windows Python 3.14，
  在不可变副本中真实执行
  `python -B -m unittest tools.session_coordinator.tests.test_validation_copies -v`。
- 副本作业 `f7c724fce2514f5ea6c9d92f7310bb1f` / run
  `3737473ea09a49e99efede53a05487f9` 以 exit code `0` 完成，`33/33` 项为
  `OK`；受管清单哈希为
  `27ead47a5d49005d4564f3ba3db7fd30cd208d3423598d227da30ac3c66b3a41`。
- 该票据只覆盖 validation-copies 下层套件；workspace-copy 全量套件仍未执行，
  因此本 failure 继续保持 open，不复用该子集作为完整验收或 closeout 证据。

## 修复结果与回传

- 根因：The sealed immutable source manifest omitted tools/session_coordinator/validation_copy_diagnostics.py, which workspace_copy imports, so the copy failed with ModuleNotFoundError before the requested regression modules could collect.
- 架构修复：The coordinator now seals the complete reviewed import closure, including validation_copy_diagnostics.py, in the immutable validation copy; no alias, shim, fallback, or test bypass was added.
- 验证：Managed Windows Python 3.14 ticket 3331e178a3134441bb9b7c8d86d89b19, copy/job 10ca14b5f7684f8890b92bb99cdcc7ad, run 3331e178a3134441bb9b7c8d86d89b19, source manifest 200895f67ddafd3e3ff3a6a9fb2d8ca8334068d66a26e7b4e95e8c4b78d2eed4; python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v; 150 tests ran and OK.
- 回传：Coordinator generated the canonical fixed-* and return artifacts and atomically removed the original child failure record after the complete immutable-copy replay.
