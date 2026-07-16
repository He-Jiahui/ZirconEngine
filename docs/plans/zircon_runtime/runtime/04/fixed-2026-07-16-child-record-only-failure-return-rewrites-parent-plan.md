---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: child-record-only-failure-return-rewrites-parent-plan
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_failures.FailureGraphTests.test_child_record_only_return_moves_fixed_artifact_without_writing_parent_plans
resolved_at: 2026-07-16
---


# Coordinator01：child_record_only 回传错误写入父计划

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行者：`runtime04-compound-shader-persisted-reference-20260715`
- 来源执行切片：已接受的 Runtime04 修复进行 failure return。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Runtime04 和 Render18 的 handoff 都声明 `plan_link_mode: child_record_only`；协调器应只在编号子计划目录移动并保留状态，而非修改共享父计划。

## 失败现象与复现证据

旧 `FailureGraphService.return_fixed()` 无条件要求 origin/fixing 父计划包含 failure Markdown 链接。对 `child_record_only` handoff，受管 return 因 `handoff_link_missing` 拒绝，即使架构修复、验证和受管 Cargo 已全部符合门禁。

## 最低共享层根因

回传逻辑忽略 frontmatter 的 `plan_link_mode`，始终执行 `_replace_handoff_link()` 并写回两个全局计划文档。这同时违背“全局计划不写入”的协作规则，并把两个父计划变为并行会话写热点。

## 架构修复验收

- `child_record_only` return 将 `failure-{date}-{summary}.md` 移为来源子目录的 `fixed-{date}-{summary}.md`，不写 origin/fixing 父计划。
- 修复者子计划目录生成普通 status receipt，保留指向 fixed artifact 的相对链接与摘要；receipt 不是新的 handoff lifecycle。
- 普通带父计划链接的 handoff 继续保持原有双相对链接回写和原子回滚行为。

## 禁止临时方案

- 不得为了回传而在 Runtime04、Render18 或其他来源父计划添加虚假链接。
- 不得在修复者目录复制第二份 `fixed-*` handoff，避免重复 lifecycle。
- 不得把 child-only handoff 降级为普通 handoff 或略过 failure return。

## 修复结果与回传

- 根因：FailureGraphService.return_fixed 无条件更新 origin/fixing 父计划的 handoff 链接，忽略 child_record_only frontmatter，因此把协作父计划变成写入热点并拒绝合法 Runtime04 return。
- 架构修复：child_record_only 分支只将 canonical failure 移为 origin child 的 fixed artifact，并在 fixing child 写普通 status receipt 与相对链接；标准 linked handoff 保留既有双向链接和回滚路径。
- 验证：focused regression test test_child_record_only_return_moves_fixed_artifact_without_writing_parent_plans 已通过；本次 historical-milestone-manifest-integrity child-only 回传已实际创建 origin fixed artifact，未修改任一父计划。
- 回传：Runtime04 的 child-only 回传基础设施已恢复；其独立 virtual-geometry failure 仍保持 open，须由 Plugins13 owner 继续处理。
