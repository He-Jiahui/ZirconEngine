# Git 与企业微信模块前缀边界纠正

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M5
Status: completed
Files: [".codex/skills/zircon-project-skills/close-session-goal-milestones/SKILL.md", ".codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.Tests.ps1", ".codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.ps1", "docs/cli-and-tooling/local-session-coordinator.md", "docs/cli-and-tooling/session-coordinator-milestone-workflows.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-12-commit-and-wecom-module-scope-correction.md", "tools/session_coordinator/git_finalize.py", "tools/session_coordinator/notifications.py", "tools/session_coordinator/tests/test_git_finalize.py", "tools/session_coordinator/tests/test_notifications.py", "tools/session_coordinator/tests/test_workflow_commit.py", "tools/session_coordinator/workflows/milestones.py"]

## Scope delivered

- Git finalizer and milestone workflow now preserve a plain Conventional Commit such as `fix(workflow): correct notification scope`; a leading `【module】` is rejected instead of being inserted or normalized.
- The module is derived from the registered numbered plan parent folder only when formatting the WeCom notification. The first line is `核心内容摘要：【{module}】<中文摘要>`; the fourth line contains the real SHA and unprefixed Git subject.
- Notification module values are restricted to safe plan-folder characters, preventing newline or bracket injection into the four-line message.
- The closeout skill, validator, examples, and operator documentation use the same boundary. Existing historical commits are not rewritten.

## Fresh testing evidence

- `python -m unittest -v tools.session_coordinator.tests.test_git_finalize tools.session_coordinator.tests.test_notifications tools.session_coordinator.tests.test_workflow_commit`: 37 passed in 292.031 seconds.
- Post-refactor focused Python gate covering Git formatting, prefix rejection, safe module formatting, and end-to-end commit/notification separation: 5 passed in 24.559 seconds; `py_compile` passed for all three production modules.
- Focused real closeout fixtures: plain Conventional Commit accepted; `【feature】feat(runtime): ...` rejected with `invalid_commit_prefix`; 2 passed in 37.3 seconds.
- The complete Pester closeout suite exceeded its 10-minute command limit without producing an early failure and is therefore recorded as timed out, not passed. The changed prefix contract was separately exercised through the same fixture and validator implementation.

## Review

- Confirmed the WeCom webhook remains environment-only and no URL/key is passed into Git or notification persistence.
- Confirmed no history rewrite, branch, worktree, stash, or broad staging operation was used.
- Confirmed unrelated shared-workspace changes remain outside this correction manifest.
