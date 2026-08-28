---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: integrated-bootstrap-tracked-test-ephemeral-dependency
origin_plan: docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/15
fixing_child_dir: docs/plans/optimize/zircon_tooling/15
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/tests/test_tracked_tests_do_not_depend_on_codex_sessions.py
tests:
  - tools/tests/test_tracked_tests_do_not_depend_on_codex_sessions.py
---

# Tooling15 integrated bootstrap tracked test ephemeral dependency

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`
- 来源执行切片：tracked test reproducibility boundary sweep
- 修复责任计划：`docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`
- 交接原因：Tooling15 owns MVP bootstrap evidence and fixture lifecycle tests.

## 失败现象与复现证据

`test_tracked_tests_do_not_read_ephemeral_codex_sessions` reports one tracked
violation at `tools/tests/tooling15-integrated-bootstrap.Tests.ps1:5`. That
test reads `.codex/sessions/tooling15-integrated-bootstrap.ps1`, an untracked,
Session-specific execution artifact.

## 最低共享层根因

A one-time bootstrap verification was committed as a permanent repository test
even though its subject remained in ephemeral Session state. A fresh clone
cannot reproduce the test and another Session can mutate or remove its input.

## 架构修复验收

- Remove the tracked test that reads the Session-only bootstrap artifact.
- Keep the ephemeral bootstrap and historical fixed record untouched.
- Preserve the repository-wide guard against `.codex/sessions` dependencies.
- Prove the tracked test dependency audit has no violations.

## 禁止临时方案

- Do not copy a Session bootstrap into production solely to retain this test.
- Do not weaken or exclude the `.codex/sessions` dependency guard.
- Do not delete or alter another Session's bootstrap artifact.

## 修复结果与回传

The non-reproducible tracked Pester test is removed while the ephemeral
bootstrap and historical fixed record remain untouched. Session-dependency and
personal-plan guards pass 2/2, and a direct scan finds no `.codex/sessions`
dependency under `tools/tests`. The scoped diff gate is clean. The exact-two
coordinator finalizer must reproduce the guards without foreign worktree
inputs.

Open state: `source_validated / failure_return_pending`.
