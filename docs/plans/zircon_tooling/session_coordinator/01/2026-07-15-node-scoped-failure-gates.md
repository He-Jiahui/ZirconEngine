---
record_kind: implementation_slice
status: in_progress
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_workflow_commit.py
  - tools/session_coordinator/tests/test_milestone_failure_scope.py
plan_sources:
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_database tools.session_coordinator.tests.test_failures tools.session_coordinator.tests.test_workflow_commit -v
  - python -m unittest tools.session_coordinator.tests.test_milestone_failure_scope -v
  - python -m compileall -q tools/session_coordinator
---

# Workflow node scoped Failure gate

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `IMPLEMENTED / LOCAL GATE GREEN` | 2026-07-15 | schema 34 为 `failure_nodes` 增加可选 `origin_workflow_node` 与查询索引；importer 只从 frontmatter 读取结构化 node，非法值诊断后按 legacy plan-wide 保存；Failure service 支持 slice exact、父 milestone 聚合 child slices、fixing-plan priority 与 legacy plan-wide。定向 RED 后，数据库/importer/workflow 回归 fresh `46/46` 通过；补充 v33 数据保留、索引与幂等升级后 `test_database` `14/14`，invalid node 诊断/fallback `1/1`，`compileall` 通过。 |
| `RUNTIME / SCHEMA 34 LOADED` | 2026-07-15 | 受控重载后的 coordinator instance `ce4b383873534671b6d53c8e27bc80f2` 报告 schema `34`；重载前唯一 Frameworks05 framebuffer 作业未被本 Session 终止，最终由其受管 60 分钟上限记录为 released/exit `124`。 |
| `OPEN / FINAL GIT GUARD` | 2026-07-15 | 独立复核发现 workflow gate 与 fingerprint 已按 node scope 收敛，但 `GitFinalizeService.commit_milestone` 的 staging 前和 `update-ref` 前守卫仍 plan-wide。production-like 新测试中 explicit finalize plan-wide `1/1` 通过，其余 milestone scope `7/7` 精确 RED；在 finalizer hard-cut、完整回归、独立复核和 Editor02 upward replay 前，本记录保持 `in_progress`，不得返回 lifecycle。 |
| `IMPLEMENTED / FINALIZER GREEN` | 2026-07-16 | `commit_milestone` 已 hard-cut 为必填、非空且 canonical 的 workflow-node tuple；`MilestoneWorkflowService.commit` 只解析一次 scope，并让 initial context、锁内 precommit context 与 finalizer staging/update-ref 两次 Failure guard 复用同一 tuple；explicit finalize 继续 plan-wide。fresh `test_milestone_failure_scope` `11/11`、`test_workflow_commit` `22/22`、`test_git_finalize` `40/40`、数据库/Failure/deferred/scope 组合门 `41/41`、`py_compile` 与 scoped diff-check 全部通过；独立只读 review 为 `P0=0 / P1=0 / P2=0`。共享 `git_finalize.py` 仍含 active artifact-governance owner 的 pending 闭包，因此在 owner 原子提交、schema 35 重载和 Editor02 upward replay 前继续保持 `in_progress`。 |

## 已实现边界

- slice 只读取与自身 node key 相同的 origin Failure；父 milestone 使用自身 key 与直接 child slice keys 的闭包。
- fixing plan 仍读取全部 open Failure，不能借 node scope 绕过 Failure Priority Gate。
- 缺少结构化 node 的历史记录保守按 plan-wide 处理；不从 Markdown 正文或 summary slug 推断归属。
- context failure revision 与 gate evidence 使用同一适用集合，相关 Failure 变化会使已有 evidence 失效。

## 剩余门禁

- [x] `GitFinalizeService.commit_milestone` 使用无默认、非空、canonical 的 `failure_workflow_node_keys` 参数。
- [x] milestone 专用 guard 在同一 Git mutex 内的 staging 前与 `update-ref` 前使用同一 immutable scope；explicit finalize 保持独立 plan-wide。
- [x] production-like 测试覆盖 unrelated sibling 放行，own/parent/legacy/fixing 阻断、非法 scope，以及两次锁内检查期间新增 Failure。
- [ ] Coordinator01 共享源码由 active owner 形成受管原子提交并加载到 schema 35 daemon。
- [ ] 对 Editor02 M1.3 重新执行 prepare、managed validation、independent review、commit 与 Failure return。
