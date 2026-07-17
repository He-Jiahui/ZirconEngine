# Coordinator Flow Efficiency M2 Implementation Plan

**Goal:** Make Enterprise WeCom delivery an observable post-commit side effect that cannot make a durable milestone commit appear to fail.

**Scope:** `MilestoneWorkflowService` and the existing WeCom notification ledger only. This slice does not change admission, Cargo ownership, lifecycle supervision, Git validation gates, or another Session's maintenance work.

## Design

The commit intent is reconciled before any WeCom formatting or delivery runs. Delivery already has an idempotency key of `(commit_sha, channel)` and stores normal provider failures. M2 closes the remaining caller-visible gap: any unexpected formatting, repository-query, or notification-service exception after the durable ref update is recorded as an `unknown` delivery outcome and returned with the successful commit result. It must not be rethrown as a milestone failure, retried implicitly, or roll back/alter the commit.

## Test-first slices

- [x] Add a failing workflow-commit regression that makes post-commit notification preparation throw. Assert `commit()` still returns its new commit SHA, HEAD advances, and the returned notification outcome is visible as non-delivered.
- [x] Add a notification-ledger helper that creates a bounded, sanitized `unknown` record for a post-commit preparation failure without invoking the webhook and without retry eligibility.
- [x] Route the post-commit WeCom block through that helper; preserve the existing successful and provider-failed delivery behavior.
- [x] Run `python -m unittest tools.session_coordinator.tests.test_workflow_commit tools.session_coordinator.tests.test_notifications -v`, then `python -m compileall -q tools/session_coordinator` and `git diff --check`.

## Acceptance

- [x] A committed milestone remains committed when WeCom formatting/delivery preparation fails.
- [x] The failure is represented by a sanitized, idempotent notification attempt, not hidden and not retried.
- [x] Existing successful/failed WeCom calls remain post-commit and preserve their one-attempt semantics.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| M2 | 提交后 WeCom 失败降级与一次性通知账本 | 已完成 | 2026-07-17 | 新增格式化失败与账本幂等回归；`test_workflow_commit` 26/26、`test_notifications` 6/6、`compileall`、`git diff --check` |
