---
record_kind: milestone_output
status: accepted
completed_at: 2026-07-16
milestone: M3
plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_sources.py
  - tools/tests/test_runtime_plan_status_canonical_archive_sources.py
tests:
  - python -m unittest tools.tests.test_runtime_plan_status_canonical_archive_sources -v
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
---

# Runtime15 M3：Plan-status canonical archive source 收敛

## 状态

- 状态锚：`runtime_15_plan_status_canonical_archive_source_accepted`。
- `runtime_numbered_archives(...)` 同时读取 active child 与 canonical `_archive` child，二者均返回 repo-root-relative 路径。
- `markdown_repo_link_targets(...)` 解析并规范化父计划中的真实 Markdown target；只有精确 repo-root path 可满足 routing，纯文本同名和错误 active target 均拒绝。
- 已归档记录不再因 active copy 删除而从 plan-status 审计证据中消失；不恢复旧文件，也不增加 fallback 副本。

## TDD 与验证

- RED：focused suite 2/2 首次失败，分别证明 source resolver 只返回 active child，以及 Runtime05 status gap 仍存在。
- GREEN：review 修复后的 focused suite 3/3，`status_table_gaps=[]`、Runtime02 generated index 缺口为空、Runtime10 behavior-status doc 缺口为空。
- Review 修复：新增 correct canonical / wrong active / bare filename 三组 link-target 回归，移除 basename-only 误通过。
- 直接审计仅剩 Runtime01/10/15 的 `last_refined` 元数据风险。
- 完整 Runtime structure audit 的 plan-status 风险已从四项降为这一项；其余 Root/Operation/Navigation 风险来自外来并行拓扑，不在本切片。
- 交叉 archive suite 的 Runtime15 single-owner 通过；仅 `last_refined_violations` 断言保持 RED，作为下一纯元数据切片。
- 独立 review：critical 0、important 0。
- 受管提交：本记录随同 boundary/source/test/module-doc 五文件 exact slice 提交；foreign `test_runtime_plan_status_archive_ownership.py` 明确排除。

## 范围

- 不修改外来删除的 active output records。
- 不改 Runtime02 root public module count、Runtime10 operation ABI module inventory 或 Runtime14 navigation file count。
- 不把历史 archive 当成 parent plan；parent 继续只拥有 current routing/status。
