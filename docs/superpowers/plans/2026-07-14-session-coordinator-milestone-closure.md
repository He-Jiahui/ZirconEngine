# Session Coordinator Milestone Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make accepted child-plan milestones use one local coordinator path for managed validation, commit, progress recording, WeCom delivery, and target cleanup.

**Architecture:** Reuse `MilestoneWorkflowService` as the sole milestone commit authority. Add a narrow command-line bridge for the existing prepare/validate/commit/goal-close actions, then update closeout and Cargo skills to use that bridge. A PreToolUse guard prevents future raw artifact-producing Cargo shell commands from bypassing the coordinator.

**Tech Stack:** Python 3.14 coordinator, PowerShell command wrapper, Codex JSON Hooks, SQLite workflow records, existing WeCom script.

---

## File Structure

- `tools/session_coordinator/cli.py`: Parse the `milestone` command family and send typed requests.
- `tools/session_coordinator/client.py`: Execute existing local controlled-action preview/confirm calls for the command bridge.
- `tools/session_coordinator/control_plane/actions/executor.py`: Return the imported workflow run identifier needed by the local command bridge.
- `tools/session_coordinator/workflows/milestones.py`: Keep the existing commit/notification authority; add only narrowly-scoped helpers if the command bridge needs a stable preparation result.
- `tools/session_coordinator/tests/test_workflow_commit.py`: Cover the command-to-service milestone commit path, workflow attempt, and exactly-once notification behavior.
- `tools/session_coordinator/workspace_copy.py`: Remove terminal managed validation copies after evidence import and retain failed cleanup for retry.
- `tools/session_coordinator/tests/test_workspace_copy.py`: Cover terminal validation-copy deletion and retry-safe cleanup state.
- `tools/session_coordinator/tests/test_cli.py` or an existing CLI test module: Cover argument validation and the four command requests.
- `.codex/hooks/pre_tool_use_cargo_guard.py`: Reject unmanaged artifact-producing Cargo shell commands and append a sanitized local diagnostic.
- `.codex/hooks.json`: Register the new `PreToolUse` Bash hook without changing existing session-sync events.
- `tools/session_coordinator/tests/test_cargo_guard.py`: Cover allowed read-only Cargo commands, rejected raw builds, and approved validator invocations.
- `.codex/skills/zircon-project-skills/close-session-goal-milestones/SKILL.md`: Replace the obsolete `finalize --milestone` procedure with the explicit milestone command sequence.
- `.codex/skills/zircon-dev/validation/SKILL.md` and `manual-commands.md`: State that build-producing Cargo calls must use the coordinator-aware validator or milestone validation operation.
- `docs/cli-and-tooling/local-session-coordinator.md`: Document the local command sequence, notification timing, guard limitation, and thirty-second cleanup retry behavior.

## Milestone M1 — Expose the Existing Milestone Authority

### Implementation slices

- [ ] Add `zircon-session milestone prepare --session-id <id> --milestone M<n>`.
  - Read the Session plan path from `SessionService`.
  - Import/activate the topology through `TopologyImporter.import_plan`.
  - Bind an exact current attributed manifest through `MilestoneWorkflowService.bind_manifest`.
  - Return `runId`, `topologyVersionId`, milestone key, and bound paths.

- [ ] Add `zircon-session milestone validate --session-id <id> --run-id <id> --milestone M<n> --template <template>`.
  - Reuse the existing `ValidationStartParameters` semantics and workspace-copy service.
  - Reject unknown templates, a foreign run owner, a missing manifest, or a non-managed target path.
  - Return the validation run and temporary copy identifiers so the worker can wait for evidence before commit.

- [ ] Add `zircon-session milestone review --session-id <reviewer> --executor-session-id <id> --run-id <id> --milestone M<n> --critical-count <n> --important-count <n> --summary <text>`.
  - Reuse the existing independent-review action and reject the executor Session as its own reviewer.
  - Refresh gate evidence after managed validation completes.

- [ ] Add `zircon-session milestone commit --session-id <id> --run-id <id> --milestone M<n>`.
  - Route to `MilestoneWorkflowService.commit`; do not call `GitFinalizeService.commit_milestone` directly.
  - Preserve its current atomic order: gate recheck, scoped commit, workflow-attempt reconciliation, then `WeComNotificationService.notify_once`.
  - Return SHA, shortstat, accepted milestone key, and notification status without exposing webhook configuration.

- [ ] Add `zircon-session milestone close-goal --session-id <id> --run-id <id>`.
  - Route to `MilestoneWorkflowService.close_goal`.
  - Reject incomplete milestones, open applicable failures, dirty owned scope, pending patches, and live leases.

### Testing stage

- [ ] Add focused Python tests that prove a successful CLI milestone commit produces one accepted workflow attempt and one notification reservation for the commit SHA.
- [ ] Add tests that a failed gate or failed finalizer creates neither an accepted attempt nor a notification.
- [ ] Run `python -m unittest tools.session_coordinator.tests.test_workflow_commit tools.session_coordinator.tests.test_action_execution -v`.
- [ ] Record the accepted command output and commit SHA in the owning child-plan output record.

## Milestone M2 — Prevent Unmanaged Builds and Close Temporary Artifacts

### Implementation slices

- [ ] Implement `.codex/hooks/pre_tool_use_cargo_guard.py`.
  - Parse the Hook JSON defensively and inspect only the Bash command text.
  - Reject standalone `cargo build`, `check`, `test`, `run`, `bench`, `clippy`, `doc`, and `clean`, including `cargo.exe` and `& cargo` PowerShell forms.
  - Allow `cargo metadata`, `tree`, `fmt`, the coordinator-aware `validate-matrix.ps1` entry point, and coordinator client commands.
  - Append only timestamp, repository-relative context, normalized subcommand, and denial reason to a state-root JSONL log; never log transcript contents, tokens, or webhook data.

- [ ] Register the guard in `.codex/hooks.json` for `PreToolUse` with matcher `Bash` and the existing Windows git-root resolution convention.

- [ ] Ensure the managed validation completion path removes the temporary validation copy after terminal result import unless the record explicitly identifies a compatible reusable Cargo pool.
  - Preserve the existing `finally` release in `validate-matrix.ps1`.
  - Preserve the daemon's thirty-second retry for failed ephemeral target deletion.

### Testing stage

- [ ] Add unit tests for raw build rejection, metadata allowance, validator allowance, malformed Hook input safety, and sanitized diagnostic output.
- [ ] Run `python -m unittest tools.session_coordinator.tests.test_cargo_guard tools.session_coordinator.tests.test_cargo_jobs -v`.
- [ ] Invoke the validator in `-DryRun` mode and confirm the acquired target is under a D/E/F managed root and is released.
- [ ] Record live target and cleanup counts after the test; do not use historical rows as acceptance evidence.

## Milestone M3 — Make the Required Path the Default Session Behavior

### Implementation slices

- [ ] Update `close-session-goal-milestones` so its Milestone sequence is prepare → managed validation → milestone commit → inspect notification result; its Goal sequence adds `close-goal` after the final accepted milestone.
- [ ] Update validation skills and the local coordinator guide with the raw Cargo prohibition, guard limitation, cleanup behavior, and direct command examples.
- [ ] Update the current Session note with the new active workflow and any foreign staged-scope restriction.

### Testing stage

- [ ] Run the skill validator for every edited project skill.
- [ ] Run `python -m compileall -q tools/session_coordinator` and the focused test groups from M1 and M2.
- [ ] Run `git diff --check` for only the coordinator, hook, skill, and documentation paths owned by this Session.
- [ ] Recheck the Failure graph before closeout. If a coordinator-plan failure exists, enter `resolving_failure` and return it before ordinary work.

## 状态与产出记录

| 里程碑 | 切片/阶段 | 状态 | 日期 | 证据 |
| --- | --- | --- | --- | --- |
