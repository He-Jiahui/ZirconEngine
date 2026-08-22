---
handoff_kind: fixed
status: fixed
created_at: 2026-08-23
summary_slug: fixed-content-section-preservation
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_child_record_only_return_preserves_required_sections_after_result -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_return_rejects_duplicate_real_result_sections -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures -v
resolved_at: 2026-08-23
---

# Coordinator01: fixed return truncates required trailing sections

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：compile-time resource closure lifecycle return.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Failure artifact transformation and validation are Coordinator01-owned.

## 失败现象与复现证据

The production `failure return` for `compiletime-resource-closure` generated a
fixed artifact without `## 禁止临时方案`. The canonical validator reported one
target diagnostic until the generated artifact was repaired. A direct call to
`FailureGraphService._fixed_content` also proves that a required section placed
after `## 修复结果与回传` disappears from the output.

## 最低共享层根因

`FailureGraphService._fixed_content` splits the body at the result heading and
rebuilds only the prefix plus the new result fields. It treats the result section
as an implicit end-of-file marker even though the handoff schema does not require
that ordering.

## 架构修复验收

- Replace only the body of the single real `## 修复结果与回传`, ending at the
  next real level-two heading or the end of the artifact; fenced and indented
  code are not headings, and duplicate real result sections fail closed.
- Preserve every later heading and byte-equivalent section body in its original
  order.
- Keep existing last-section returns, child-record receipts, and rollback behavior.
- The focused RED and full `test_failures` suite pass.

## 禁止临时方案

- Do not reorder or manually patch every source handoff before return.
- Do not weaken the validator or make required headings optional.
- Do not special-case `## 禁止临时方案`; preserve arbitrary later sections.

## 修复结果与回传

- 根因：FailureGraphService._fixed_content treated the repair-result heading as an implicit end-of-file marker and rebuilt only the prefix plus result, discarding required trailing sections.
- 架构修复：Parse one real top-level repair-result H2 with fence-aware duplicate rejection, then replace only its body by raw offsets so every later heading and body remains byte-equivalent.
- 验证：Successor 4cc85204c80b4fed85d307cd24894ca1 loaded commit 8bf16edbf0; focused fence, container, suffix, and duplicate tests plus the full test_failures suite passed 28/28; canonical target diagnostics are zero.
- 回传：Coordinator01 fixed return generation now preserves arbitrary later sections and hard breaks without validator weakening or heading reordering.
