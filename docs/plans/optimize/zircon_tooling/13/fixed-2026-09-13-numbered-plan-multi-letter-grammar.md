---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-10
summary_slug: numbered-plan-multi-letter-grammar
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/optimize/zircon_tooling/13
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/plans.py
  - tools/session_coordinator/tests/test_plans.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_plans -v
  - python -B -m unittest .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py -v
resolved_at: 2026-09-13
---

# Numbered plan multi-letter grammar failure

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：current failure-audit schema diagnostics for the Runtime136 composition handoff
- 修复责任计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 交接原因：Tooling13 owns the shared numbered-plan identity grammar and the handoff validator that must agree with the repository's 99za-99zz plan sequence.

## 失败现象与复现证据

The repository contains formal review plans whose identifiers extend the two-digit sequence with
multiple ASCII letters, including `99zk` and the surrounding `99za`-`99zz` plans. The coordinator
and handoff validator still accept only one optional letter, so the Runtime136 failure handoff is
reported with `fixing_plan must name a numbered child plan`. The coordinator and validator also
disagree about the derived child directory for that handoff, preventing immutable validation from
reaching the underlying Runtime136 source test.

## 最低共享层根因

`PLAN_DEFINITION`, `NUMBERED_CHILD_DIR`, and the handoff validator's `PLAN_NAME` encode a one-letter
suffix instead of the repository's actual multi-letter ASCII identifier grammar. The duplicated
regular expressions allow the drift to persist until a handoff references a later sequence entry.

## 架构修复验收

- Coordinator and handoff validation share one exact two-digit plus one-or-more ASCII-letter grammar.
- Numeric-only and multi-letter identifiers normalize to lowercase, while non-ASCII, malformed-width,
  and non-letter suffixes remain rejected.
- A handoff using `99zk` derives the `99zk` child directory and imports with a stable lifecycle key.
- Focused coordinator and validator regressions pass, and the original Runtime136 artifact can then
  be corrected without changing its factual evidence.

## 禁止临时方案

- Do not special-case `99zk`, add an allow-list, weaken Unicode validation, or rewrite the Runtime136
  failure to a different owner merely to suppress the diagnostic.
- Do not delete the original reproduction, bypass immutable snapshots, or claim Cargo validation here.

## 修复结果与回传

- 根因：The coordinator parser and handoff validator used duplicated one-letter suffix expressions, so valid multi-letter numbered plan identifiers such as 99zk were rejected or mapped to the wrong child directory.
- 架构修复：Converged PLAN_DEFINITION, NUMBERED_CHILD_DIR, and PLAN_NAME on one ASCII two-digit plus zero-or-more-letter grammar with lowercase normalization and strict rejection of malformed, non-ASCII, and non-letter suffixes; no identifier-specific allow-list or compatibility shim was added.
- 验证：Managed ticket 83cdac6e60e946f58a797cac337385ee passed the combined coordinator and handoff validator suite (24 tests) in an immutable copy; ticket 7ec43d95209c498f9900e86ca1ab8255 passed the exact coordinator parser suite (5 tests). Current source hashes for plans.py, test_plans.py, validate_plan_failure_handoffs.py, and its test match the sealed manifests; repository handoff validator reports 818 artifacts and 0 errors.
- 回传：Numbered-plan multi-letter grammar is fixed and returned. The original Runtime136-related handoff evidence is preserved; the shared parser and validator now accept 99zk and derive its lowercase child directory.

## 2026-09-11 failure rolling repair

- The existing source repair is present in the claimed scope: coordinator `PLAN_DEFINITION`/`NUMBERED_CHILD_DIR` and the handoff validator `PLAN_NAME` now use the same ASCII two-digit plus zero-or-more-letter suffix grammar, normalize accepted identifiers to lowercase, and reject malformed or non-ASCII identifiers.
- Current local dynamic evidence (not a managed ticket) is green: `python -B -m unittest tools.session_coordinator.tests.test_plans -v` ran 5/5, and `python -B -m unittest discover -s .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts -p 'test_validate_plan_failure_handoffs.py' -v` ran 19/19. `py_compile` and scoped `git diff --check` also completed without errors.
- The stale primary `failure-roll-01a084c8-tooling13-plan-grammar-r2` was resumed with its original scope and baseline attribution. No fixed/return artifact has been created, and no lifecycle closure is implied by local tests.
- Managed request `tooling13-plan-grammar-coordinator-20260911-r1` created queued ticket `7ec43d95209c498f9900e86ca1ab8255`, but admission reported `validation_dependency_failed` on the open lifecycle `validation-ticket-dedupe-closeout-runtime-identity`, owned by the separate Tooling01 upload primary. The ticket has not produced a run or managed test result; this lifecycle remains open and waits for that dependency.
