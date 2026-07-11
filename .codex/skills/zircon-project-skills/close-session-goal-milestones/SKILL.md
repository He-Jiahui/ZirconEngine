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

## Validate and commit

Build every automatic Git subject as a plain Conventional Commit, for example `feat(workflow): complete M5 milestone`. A subject beginning with `【{module}】` is invalid. Derive the module from the registered numbered plan's parent directory only after the commit, exclusively for the WeCom summary line; never insert it into Git history.

Stage only the manifest paths. If a repository-owned skill path is intentionally covered by the blanket `.codex` ignore, use `git add -f -- <exact-path>` only for that attributed manifest entry. Then run:

```powershell
& .\.codex\skills\zircon-project-skills\close-session-goal-milestones\scripts\check-closeout.ps1 `
  -RepoRoot . -Mode $mode -SessionId $sessionId `
  -CommitMessage $message -ManifestPath $manifestPath
```

Require `status: ok`. The checker reads Session/current-hash attribution from coordinator SQLite in read-only mode and completion evidence from the registered plan; manifest claims cannot replace either source. Resolve foreign staged scope through coordination—never unstage or overwrite it.

Commit immediately through the coordinator so its Git mutex rechecks live leases, current-hash attribution, Failure state, and the exact index under one atomic boundary:

```powershell
$checkerCommand = "pwsh -NoProfile -File .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.ps1 -RepoRoot . -Mode $mode -SessionId $sessionId -CommitMessage `"$message`" -ManifestPath `"$manifestPath`""
$arguments = @("finalize", "--commit", "--milestone", "--session-id", $sessionId, "--message", $message, "--validation-command", $checkerCommand)
foreach ($path in $manifestPaths) { $arguments += @("--path", $path) }
& .\tools\zircon-session.ps1 @arguments
```

Never run a plain business-Session `git commit`. Never use `[zircon-session:*]`, checkpoint wording, an empty commit, branch, worktree, stash, or hidden version commit. Verify paths and SHA from the service result.

After each commit, invoke `wecom-push-message` once with four lines:

```text
核心内容摘要：【{module}】<中文核心摘要>
提交时间：<commit ISO time>
修改情况统计：<shortstat>
提交的commit内容：<SHA> <subject>
```

The fourth line must contain the real unprefixed Conventional Commit subject. For example, a plan under `docs/plans/zircon_tooling/session_coordinator/` uses `【session_coordinator】` only on the first line, while the Git subject remains `feat(workflow): ...`.

Never store the webhook URL in Git. If sending fails, report it; do not retry automatically and do not roll back the commit.

## Finish by mode

### Milestone

- Treat the committed child-plan evidence and commit SHA as the immutable milestone record; do not reopen the Markdown solely to append that SHA.
- Release unneeded leases and process only safe delayed patches.
- Restore/keep the Session `active`; keep the Goal active and continue the next milestone.

### Goal

- Reject incomplete plan items, applicable open Failures, or remaining Session-owned dirty paths.
- Do not create an empty final commit when the last milestone already contains all closeout changes.
- Confirm after the commit that the Session-owned scope has no unstaged difference, then process safely executable delayed patches.
- Release all Session leases, set the Session `completed` with the final SHA and completion reason, then mark the active Goal complete.
- Report foreign diagnostics without editing them.
- In one terminal report, include commit SHA/subject, verification evidence, foreign-Session diagnostics, and the WeCom result.

## Stop conditions

Stop for incomplete tests/review, missing lease or current-hash attribution, foreign staged paths, checker errors, or unresolved lower-layer failure. Milestone completion authorizes only its scoped commit.
