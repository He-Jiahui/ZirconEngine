# Session Coordinator Milestone Closure Design

## Goal

Make a ZirconEngine business Session complete a child-plan milestone through one local coordinator path: managed validation evidence, atomic scoped Git commit, persistent milestone progress, one WeCom notification, and immediate release or scheduled cleanup of temporary build artifacts.

## Problem

The current coordinator already has `MilestoneWorkflowService.commit`, which records the committed milestone and sends the required four-line WeCom message. It is only reachable through the browser controlled-action flow. The command-line `finalize --milestone` path instead calls `GitFinalizeService.commit_milestone` directly; it neither identifies an `M<n>` node nor invokes the notification service.

Codex Hooks currently wake the discovery worker only. A Hook `Stop` or a `task_completed` event is a turn lifecycle signal, not evidence that a plan milestone passed, so it must never trigger a commit or notification. The same separation explains why raw Cargo commands can bypass the coordinator and leave unmanaged target directories.

## Chosen Design

Expose the existing milestone service through explicit local command-line operations. The Session skill calls those operations at the accepted milestone boundary; the Hook continues to provide identity and liveness only.

```text
Session skill: prepare -> managed validation -> commit milestone -> close goal
                         |                        |
                         v                        v
                    Cargo lease/copy       Git mutex + workflow attempt
                         |                        |
                         v                        v
                 release + cleanup      WeCom once after real commit
```

### Milestone command contract

Add a `zircon-session milestone` command family backed by the coordinator:

- `prepare`: import/activate the registered plan topology and bind the current Session-owned manifest to the requested `M<n>`.
- `validate`: start an existing managed validation template for that run and milestone. It must use the validation-copy/Cargo-managed path, never a repository-local target directory.
- `review`: require a distinct reviewer Session to submit the independent review and refresh gate evidence after managed validation completes.
- `commit`: call `MilestoneWorkflowService.commit` with the exact `run_id` and `M<n>`. The service owns the plain Conventional Commit subject (`feat(workflow): complete M<n> milestone`), re-evaluates gates under the Git mutex, makes the scoped commit, reconciles the workflow attempt, and sends WeCom exactly once only after a real commit SHA exists.
- `close-goal`: call `MilestoneWorkflowService.close_goal` after all milestone nodes have accepted attempts and the Session is clean.

The CLI never adds a module prefix to Git; `MilestoneWorkflowService` derives the plan-folder module only for the WeCom first line. Legacy `finalize --milestone` is rejected, and a numbered-plan Session may become complete only through `close-goal`; neither route may create a commit record without a matching workflow attempt. A writable daemon also installs a local pre-commit gate so ordinary Git shell commits are rejected before they enter shared history.

### Codex and Session identity

Codex Hook events remain observational. The closeout skill uses the registered business `session_id`, plan path, `run_id`, and milestone key; it must not infer completion from a Codex turn. The hook will continue to refresh liveness and provide a local reminder when a Session has a prepared or validated milestone but no accepted commit.

### Cargo enforcement and cleanup

Add a repo-local `PreToolUse` Bash guard for Cargo artifact-producing subcommands (`build`, `check`, `test`, `run`, `bench`, `clippy`, `doc`, and `clean`). It rejects direct shell invocations and explains that the worker must use the coordinator-aware validator or validation action. Read-only Cargo commands such as `metadata`, `tree`, and `fmt` remain allowed.

The guard is a practical Codex-shell boundary, not a security boundary: it rejects raw artifact Cargo commands and direct `git commit`, logs each denial under coordinator state for later debugging, while the skills and validator remain the authoritative workflow. The approved validator already acquires a job, starts it, finishes it, and releases it in `finally`; release schedules cleanup. The daemon keeps the thirty-second retry cadence for targets whose deletion initially fails. Managed validation copies import bounded result evidence first, then delete their terminal job tree; a failed deletion remains durable `cleanup_pending` until a retry succeeds.

## Failure Handling

- A validation failure records evidence but cannot call milestone commit or WeCom.
- A commit failure preserves the worktree and emits no WeCom notification.
- A notification failure is recorded after the commit and is not retried automatically; it never rolls back Git.
- A failed target deletion stays in the live `failedCleanup` metric and is retried by the daemon every thirty seconds until removed or explicitly retained.
- If the plan owns an open `failure-*.md`, the Failure Priority Gate remains in force: only the fixing Session can resolve it before ordinary milestone progression.

## Acceptance Criteria

1. A milestone command creates an accepted workflow attempt with the same commit SHA and `M<n>` key.
2. A successful milestone commit produces exactly one WeCom attempt in the existing four-line Chinese format.
3. A normal Codex `Stop` or completed turn produces neither a commit nor a WeCom notification.
4. A direct artifact-producing Cargo shell command is rejected before execution with a persisted diagnostic.
5. The supported validator allocates targets only under the managed D/E/F roots and releases the job even on failure.
6. The web snapshot continues to display only real current target directories and live cleanup counts.
