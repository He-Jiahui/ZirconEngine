---
handoff_kind: fixed
status: fixed
created_at: 2026-09-07
summary_slug: handoff-letter-suffixed-plan-identity
plan_link_mode: child_record_only
origin_plan: docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/09c
fixing_child_dir: docs/plans/optimize/zircon_tooling/13
related_code:
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
tests:
  - python -B -m unittest discover -s .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts -p test_validate_plan_failure_handoffs.py -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures -v
  - python -B -m unittest tools.session_coordinator.tests.test_plans -v
  - scoped repository handoff validation and coordinator import for Runtime09C and Runtime09D
resolved_at: 2026-09-07
---

# Tooling13: Handoff Letter-Suffixed Plan Identity

## 来源执行者

- Origin Session: `failure-roll-01a07160-runtime09c`.
- 来源计划：`docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md`
- 来源执行切片：[Shader/material readiness identity failure registration](../../zircon_runtime/09c/failure-2026-09-07-shader-material-readiness-publication-identity.md).
- 修复责任计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 交接原因：Tooling13 owns repository skill validators and their contract with coordinator governance.
- Fixing Session: `failure-roll-01a07160-tooling13`.

## 失败现象与复现证据

On 2026-09-07 the repository handoff validator rejected the two new canonical
Runtime09C/09D handoffs with `fixing_plan must name a numbered child plan`.
The scoped result was 2 errors, 65 repository errors total. An isolated
HandoffFixture with `docs/plans/runtime/09c-shaders.md` and child `09c` reproduced
the same error and failed the assertion that this canonical plan must validate.

Coordinator `PlanRepository` already accepts exactly two digits plus an optional
ASCII letter, case-insensitively, and normalizes the child ID to lowercase.
The validator instead uses digits only and does not normalize the captured ID.
Its pre-repair SHA256 is
`2e15dbe7bb7f8c84202a85e8167cee5f31f965e9c4c3f7f6bbf52af788ef1f3d`;
source baseline is coordinator snapshot 2944, request
`42f2170393504a738f2b706934535d5d`.

## 最低共享层根因

The handoff parser's numbered-plan rule drifted from the coordinator's canonical
plan identity. Valid optimized subplans such as 09a-09d can register a Session
but fail handoff validation. This affects both open and returned records.

## 架构修复验收

- Match the existing coordinator plan grammar and lowercase child-ID policy.
- Verify numeric and letter-suffixed origin/fixing plans, open and fixed records,
  uppercase suffixes, stable lifecycle keys and malformed identifier rejection.
- Run the complete handoff validator fixture suite and coordinator FailureGraph
  import/return tests through managed Windows validation.
- Recheck the actual Runtime09C/09D handoffs and this artifact, preserving all
  unrelated repository diagnostics; import the canonical paths successfully.

## 禁止临时方案

- Do not rename valid 09c/09d plans, transfer their failures to a false owner,
  special-case these two artifacts, or suppress unrelated diagnostics.
- Do not count a queued ticket or unexecuted test filter as acceptance.

## 修复验证证据

Source snapshot 2952 contains the exact four parser/test files and this record.
Managed Windows ticket `619cb043ed934cecb2a0a292e7e9c94e`, request
`failure-roll-20260907-handoff-plan-id-2952-r2`, job
`0acece549be744fe9bf9939ba81010e6`, completed with exit 0: 59 tests passed in
36.152 seconds (19 handoff, 5 PlanRepository, 35 FailureGraph). Current source
hashes match the sealed manifest. Independent review in the existing task
`01a07160-5337-7570-a507-ed6decf2d32b` found C0/I0/M0 and independently passed
the same 59 tests.

Source review 2944 -> 2945 found C0/I0/M1: Python Unicode case folding also
accepted four non-ASCII suffixes. Both the handoff and coordinator parser now
use `re.ASCII | re.IGNORECASE`, with numeric-width, suffix and cross-parser
consistency regressions. The coordinator parser's pre-change scope is snapshot
2951, request `9a403723c4a74fd6b1ebd2e9c058eb30`.

First managed ticket `dbb1a3cad2544b628e6bee4aa009ad53`, request
`failure-roll-20260907-handoff-plan-id-2945-r1`, failed during materialization:
job `6fc0c73303fe4e189f5636c6b1e578ee`,
`validation_copy_dependency_archive_failed`, test execution 0 ms. Its dependency
roots erroneously included nonexistent `tools/__init__.py`; the corrected
request removes that path and includes the changed coordinator parser and tests.
This receipt is not passing validation evidence.

The actual Runtime09C, Runtime09D and this handoff validate with 0 scoped errors;
63 unrelated repository errors remain. Import request
`27749db49a4f4730945e2f60ecc11829` indexed this lifecycle under its canonical path,
confirmed by `failure open` receipt `98a29ebcd5e34d4da90ccfe86af824f3`.

## 修复结果与回传

- 根因：Handoff numbered-plan grammar diverged from coordinator identity; Unicode case folding additionally admitted non-ASCII suffixes.
- 架构修复：Unify exact two-digit optional ASCII-letter identifiers and lowercase child IDs in both parsers; cover open/fixed lifecycle keys and grammar parity.
- 验证：Snapshot 2952; managed Windows ticket/run 619cb043ed934cecb2a0a292e7e9c94e, job 0acece549be744fe9bf9939ba81010e6: exit 0, 59/59 tests (19 handoff, 5 PlanRepository, 35 FailureGraph), 36.152s. Four source hashes unchanged. Independent review task 01a07160-5337-7570-a507-ed6decf2d32b C0/I0/M0. Actual 09C/09D/current artifacts: 0 scoped diagnostics; 63 unrelated errors retained. Controlled rollover action 028e3a602b174caea6568a9ea95db38e succeeded and refreshed the old service validator cache.
- 回传：Accept canonical letter-suffixed plan handoffs without changing their lifecycle identity; bind exact validated parser/test files to coordinator failure closeout.
