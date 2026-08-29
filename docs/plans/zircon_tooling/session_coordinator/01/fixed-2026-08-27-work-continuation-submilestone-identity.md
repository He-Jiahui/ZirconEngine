---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-27
summary_slug: work-continuation-submilestone-identity
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/tests/test_control_snapshot.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_control_snapshot.ControlSnapshotTests.test_continuation_preserves_the_exact_submilestone_identity -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_snapshot.ControlSnapshotTests.test_continuation_preserves_implementation_context_through_nested_headings -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_snapshot -v
resolved_at: 2026-08-27
---

# Coordinator01: work continuation loses submilestone identity

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：waiting-session same-plan continuation projection
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：the failure is local to the Coordinator parser and control snapshot projection.

## 失败现象与复现证据

`WorkContinuationService` accepted a heading such as `## M1.2 - Focused work`,
but `_MILESTONE` captured only `M1`. The continuation card therefore projected a
different milestone identity from the plan that supplied the unchecked task. Two
independent submilestones such as `M1.1` and `M1.2` could collapse onto the same
advisory scope label.

The focused RED created a waiting primary Session whose only unchecked
implementation task belongs to `M1.2`. The production projection returned `M1` and
failed with the exact assertion `'M1.2' != 'M1'`. A second RED placed a task below
`### Implementation slices` and the neutral nested heading `#### Storage projection`;
the parser incorrectly cleared the implementation context and returned no
continuation.

## 最低共享层根因

The milestone regex captured only `M` followed by the first numeric segment. Its
word boundary allowed a match immediately before `.2`, so dotted milestone suffixes
were silently discarded rather than rejected or preserved.

## 架构修复验收

- Preserve the complete `M<number>(.<number>)*` token from a valid milestone heading.
- Preserve an implementation section through neutral nested headings while keeping a
  nested testing section excluded and resuming the parent implementation context at
  the next sibling section.
- Keep heading depth, implementation-section, testing-section, unchecked-task, plan
  ownership, and advisory-only scope rules unchanged.
- Prove `M1` remains stable and `M1.2` projects exactly through the control snapshot.
- Load the committed code in a healthy successor before returning the lifecycle.

## 禁止临时方案

- Do not infer a submilestone from task text or filesystem layout.
- Do not make continuation cards mutate Session scope or bypass the required claim.
- Do not accept arbitrary dotted words as milestone identities.

## 修复结果与回传

- 根因：The continuation milestone regex captured only the first numeric segment and the parser discarded implementation ancestry at neutral nested headings.
- 架构修复：Preserve the full M<number>(.<number>)* identity and resolve the nearest semantic implementation/testing ancestor with a bounded heading stack.
- 验证：Focused continuation projections 4/4 including M1, M1.2 and M1.2.3; complete control snapshot suite 24/24 in 113.187 seconds; py_compile and scoped diff check passed; commit 9a7c88349d22fb92c99e91f31d629a1644007ab7 loaded by healthy schema68 successor 54725740965b4e9a98b047f3bd2fd364.
- 回传：Waiting-session continuation cards now preserve exact submilestone identity and nested implementation context without projecting testing tasks or mutating Session scope.
