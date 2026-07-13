---
name: close-session-goal-milestones
description: Use when a ZirconEngine Session reaches an accepted milestone or all work in its active Goal is complete and the shared main checkout must be closed out.
---

# Close Session Goal Milestones

## Overview

Treat each accepted milestone as a normal Git commit boundary. Preserve foreign paths and complete the Session/Goal only at the terminal boundary.

## Choose the closeout

- Use `Milestone` after every implementation slice and the milestone testing stage have evidence, while later milestones remain.
- Use `Goal` only when all plan items, gates, reviews, and applicable Failures are complete.
- Do not use this skill for a completed implementation slice whose testing stage has not passed.

## Shared preflight

1. **REQUIRED:** Use `cross-session-coordination`. Confirm service health, `main`, Session identity, leases, attribution, baseline, and staged paths.
2. Query the Failure graph. If an open handoff targets this plan, use `handle-plan-failure-handoffs`; fix the lowest shared architecture before closeout.
3. Before writing plan evidence, use `write-plan-output-records`. Write only the registered numbered child-plan output, never a global `docs/plans` definition or index.
4. Run the testing stage and `verification-before-completion` for the exact scope.
5. Before the first write or deletion, heartbeat, claim the path, and attribute its base/current hash. Keep that lease live through commit; a deletion without its pre-delete lease base is not owned.
6. Inventory every Session-owned dirty path, including new files omitted from the latest prompt. Classify `code`, `docs`, `tests`, `scripts`, and `untracked` in JSON. Every untracked path must also appear in one content category. Re-attribute current hashes after the final edit. Never absorb or unstage another Session's files.

## Validate, commit, and notify

Build every automatic Git subject as a plain Conventional Commit, for example `feat(workflow): complete M5 milestone`. A subject beginning with `【{module}】` is invalid. Derive the module from the registered numbered plan's parent directory only after the commit, exclusively for the WeCom summary line; never insert it into Git history.

Do not use `finalize --milestone` for a business milestone. That legacy command creates a scoped Git commit but cannot identify the `M<n>` node, record the workflow attempt, or deliver the service-managed WeCom notification.

Before the milestone action, stage no files manually. If a repository-owned skill path is intentionally covered by the blanket `.codex` ignore, it remains eligible only when it is already attributed and appears in the service-bound milestone manifest. Run the read-only closeout checker for the exact candidate scope:

```powershell
& .\.codex\skills\zircon-project-skills\close-session-goal-milestones\scripts\check-closeout.ps1 `
  -RepoRoot . -Mode $mode -SessionId $sessionId `
  -CommitMessage $message -ManifestPath $manifestPath
```

Require `status: ok`. The checker reads Session/current-hash attribution from coordinator SQLite in read-only mode and completion evidence from the registered plan; manifest claims cannot replace either source. Resolve foreign staged scope through coordination—never unstage or overwrite it.

Use this explicit service sequence. A Codex turn ending, `Stop` Hook, or idle Session is never milestone evidence and must not be substituted for any step.

```powershell
$prepared = & .\tools\zircon-session.ps1 -Json milestone prepare `
  --session-id $sessionId --milestone $milestoneId | ConvertFrom-Json
$runId = $prepared.runId

$validation = & .\tools\zircon-session.ps1 -Json milestone validate `
  --session-id $sessionId --run-id $runId --milestone $milestoneId `
  --template coordinator-actions | ConvertFrom-Json
# Wait until the managed validation copy reaches a terminal result in the coordinator snapshot.
# Do not launch cargo directly while waiting.

# A distinct reviewer Session submits the independent review after validation is recorded.
& .\tools\zircon-session.ps1 milestone review `
  --session-id $reviewerSessionId --executor-session-id $sessionId `
  --run-id $runId --milestone $milestoneId `
  --critical-count 0 --important-count 0 --summary "<review summary>"

$committed = & .\tools\zircon-session.ps1 -Json milestone commit `
  --session-id $sessionId --run-id $runId --milestone $milestoneId | ConvertFrom-Json
```

`milestone commit` rechecks live gates, Failure state, attribution, leases, and the exact manifest under the Git mutex. It records the accepted `M<n>` attempt and invokes `WeComNotificationService` exactly once after a real commit succeeds. Read the returned notification status; never manually invoke `wecom-push-message` for that same SHA.

The service message has exactly four lines:

```text
核心内容摘要：【{module}】<中文核心摘要>
提交时间：<commit ISO time>
修改情况统计：<shortstat>
提交的commit内容：<SHA> <subject>
```

The fourth line must contain the real unprefixed Conventional Commit subject. For example, a plan under `docs/plans/zircon_tooling/session_coordinator/` uses `【session_coordinator】` only on the first line, while the Git subject remains `feat(workflow): ...`.

Never store the webhook URL in Git. If sending fails, report the recorded notification failure; do not retry automatically and do not roll back the commit.

All build-producing Cargo commands must use the managed validation action or `validate-matrix.ps1`. A repo Hook rejects ordinary `cargo build`, `check`, `test`, `run`, `bench`, `clippy`, `doc`, and `clean` invocations before they create an unleased target directory. `cargo metadata`, `tree`, and `fmt` remain read-only/non-target inspections.

## Finish by mode

### Milestone

- Treat the committed child-plan evidence and commit SHA as the immutable milestone record; do not reopen the Markdown solely to append that SHA.
- Confirm the service result contains the accepted milestone key, SHA, shortstat, and WeCom attempt status.
- Release unneeded leases and process only safe delayed patches.
- Restore/keep the Session `active`; keep the Goal active and continue the next milestone.

### Goal

- Reject incomplete plan items, applicable open Failures, or remaining Session-owned dirty paths.
- Do not create an empty final commit when the last milestone already contains all closeout changes.
- Confirm after the final milestone that the Session-owned scope has no unstaged difference, then process safely executable delayed patches.
- Close the Goal through `& .\tools\zircon-session.ps1 milestone close-goal --session-id $sessionId --run-id $runId`; the service validates complete milestone attempts, open Failures, pending patches, and live leases before setting the Session complete.
- Report foreign diagnostics without editing them.
- In one terminal report, include commit SHA/subject, verification evidence, foreign-Session diagnostics, and the WeCom result.

## Stop conditions

Stop for incomplete tests/review, missing managed validation result, missing lease or current-hash attribution, foreign staged paths, checker errors, a non-terminal WeCom result, or unresolved lower-layer failure. Milestone completion authorizes only its scoped commit.
