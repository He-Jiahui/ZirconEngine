---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-31
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
- Preserve the guard against `.codex/sessions` dependencies in Git-tracked
  `tools/tests` Python and PowerShell files.
- Prove the tracked test dependency audit has no violations.

## 禁止临时方案

- Do not copy a Session bootstrap into production solely to retain this test.
- Do not weaken or exclude the `.codex/sessions` dependency guard.
- Do not delete or alter another Session's bootstrap artifact.

## 修复结果与回传

- 根因：A permanent tracked Pester test read a Session-only untracked bootstrap, so fresh clones and independent Sessions could not reproduce its input.
- 架构修复：Commit 36d47fd0e removed only the non-reproducible tracked Pester test. The retained guard now enumerates Git-tracked `tools/tests` inputs in a real repository, enumerates the already-materialized inputs in a gitless validation copy, freezes that set at import, and normalizes path case and separators without matching the Session root itself.
- 早期验证：Managed ticket `8bbeb3b99ebe41368b49a942b258581e` passed 1/1 and was accepted as evidence `f525bf29aa1c45e4b1688faff2e4fc79`; its closeout input fingerprint was `f8d775b3846e035f1e8dbaad8937f14e76f458a299b2e5b4e2286f90386de41b`.
- 支持层回归：Ticket `43ee5408e39f46468eecd72905856a88` provided the RED proof that an immutable copy has no `.git`; ticket `26a3550070994594ae33e9e52371fb13` then failed closed at `validation_copy_attribution_stale` after the reviewed source changed, before test execution.
- 验证：Managed ticket/run `2b993f52bd394d039361d81edc433dad` passed 5/5 from copy `0e63f99cec7041a3be396a5879c1341b`, immutable input manifest `d156c75aed71fd7c3bd92c39737fb747580e7e7dd409baeeaf35a919fe2243b4`, and source manifest `dc8ede92cf51d1f02ff4559b797fdee196aeb6eccb9ba1501ddd8b0f53c23ef6`.
- 回传：Tooling15 tracked-test reproducibility is restored; the source-only ephemeral dependency failure is fixed and its acceptance gate may resume.
