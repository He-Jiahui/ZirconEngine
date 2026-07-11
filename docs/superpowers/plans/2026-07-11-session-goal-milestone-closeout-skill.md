# Session Goal Milestone Closeout Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a repository-local skill that closes either a completed milestone or an entire Session Goal, commits every accepted milestone safely on shared `main`, and preserves coordinator, Failure, evidence, and WeCom boundaries.

**Architecture:** Add one focused child skill under `zircon-project-skills` plus a read-only PowerShell closeout checker. The skill owns completion semantics and orchestration; the existing Session coordinator remains canonical for status, leases, attribution, Failure priority, and delayed patches. The checker validates an explicit manifest and staged scope without mutating Git, service state, or business files.

**Tech Stack:** Markdown/YAML Codex skills, PowerShell 5+/7, Pester 5-compatible tests, Git CLI, existing `tools/zircon-session.ps1`, skill-creator validators.

**Approved design:** `docs/superpowers/specs/2026-07-11-session-goal-milestone-closeout-design.md`

**Repository constraints:** Work directly in the shared `main` checkout. Do not create a branch, worktree, stash, hidden checkpoint commit, Session-tag commit, repo-local Cargo target, or webhook configuration file. Stage and commit only the explicit skill milestone scope.

---

## File Structure

- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/SKILL.md`: concise trigger and Milestone/Goal closeout workflow.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/agents/openai.yaml`: UI metadata and explicit `$close-session-goal-milestones` default prompt.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.ps1`: read-only manifest, Git index, status, message, and secret checks.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/read-closeout-evidence.py`: read coordinator Session/attribution evidence through a read-only SQLite connection.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/test_read_closeout_evidence.py`: prove live evidence is independently derived and the coordinator database remains byte-identical.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/seed-closeout-test-state.py`: seed isolated temporary repositories with real coordinator state for tests without a production fixture bypass.
- Create `.codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.Tests.ps1`: temporary-repository positive and negative tests.
- Modify `tools/session_coordinator/{git_finalize.py,server.py,cli.py,failures.py}`: add the service-owned atomic Milestone commit path, live-lease guard, and origin/fixer Failure acceptance.
- Modify `tools/session_coordinator/tests/test_git_finalize.py`: prove atomic Milestone commit, active-Session preservation, validation-under-mutex, lease enforcement, and deletion ownership.
- Modify `docs/cli-and-tooling/local-session-coordinator.md`: document milestone commit architecture and commands.
- Modify `.codex/skills/zircon-project-skills/SKILL.md`: route completion/closeout work to the new child skill.
- Modify `.codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md`: add the skill to the shallow tree and summary catalog.

## Milestone M1: Build and deploy closeout discipline

### Goal

Provide one tested, discoverable project skill that creates normal milestone commits without absorbing another Session's work and closes the overall Goal only after aggregate acceptance.

### In-scope behaviors

- Distinguish `Milestone` from `Goal` completion.
- Require milestone implementation slices and testing-stage evidence before commit.
- Inventory code, docs, tests, scripts, and previously untracked files separately.
- Validate explicit path scope, current `main`, Session identity/status, index isolation, commit-message policy, and credential patterns.
- Commit every accepted milestone and send one four-line WeCom notification after each successful commit.
- Keep the Session active after a nonterminal milestone.
- Release leases and complete Session/Goal only at terminal Goal closeout.
- Preserve foreign dirty/untracked paths and report foreign plan/Failure diagnostics without editing them.

### Dependencies

- `tools/zircon-session.ps1` and the schema-v13 local coordinator are available.
- `cross-session-coordination`, `handle-plan-failure-handoffs`, `write-plan-output-records`, and `verification-before-completion` remain canonical for their own boundaries.
- The approved design document is committed at `e43bf555` or a descendant.

### Implementation slices

- [x] **M1.1 Capture RED pressure evidence:** ask a fresh agent without the new skill to close a milestone in a dirty shared-main scenario containing foreign untracked files, a missing test script, pressure to use a Session-tag commit, and a failed WeCom notification. Record which required boundaries are omitted or violated; do not let the scenario mutate the real repository.
- [x] **M1.2 Scaffold the focused skill:** run `skill-creator/scripts/init_skill.py close-session-goal-milestones --path .codex/skills/zircon-project-skills --resources scripts --interface ...`, remove generated placeholders, and keep the root `SKILL.md` below 500 words unless a concrete workflow requirement needs more.
- [x] **M1.3 Write checker tests before implementation:** create Pester cases using generated temporary repositories and real seeded coordinator state. The tests must expect failure for a missing manifest path, a foreign staged path, omitted untracked file, non-`main`, invalid Session/checkpoint commit message, staged webhook/maintenance token, and incomplete Goal evidence; they must expect success for isolated Milestone and Goal manifests.
- [x] **M1.4 Implement the read-only checker:** accept `-RepoRoot`, `-Mode Milestone|Goal`, `-SessionId`, `-CommitMessage`, and `-ManifestPath`. Resolve every path under the repository, require the manifest categories `code`, `docs`, `tests`, `scripts`, and `untracked`, compare the exact staged set against live coordinator attribution and registered-plan evidence, scan only staged added lines, and emit JSON without calling `git add`, `git commit`, status mutation, lease mutation, or WeCom.
- [x] **M1.5 Write the skill workflow:** define the shared preflight, Milestone path, Goal path, failure behavior, no-empty-commit rule, ordinary Conventional Commit rule, four-line WeCom format, no-retry rule, coordinator attribution/lease rules, and foreign-work preservation rule. Require `verification-before-completion`, `cross-session-coordination`, `write-plan-output-records`, and `handle-plan-failure-handoffs` only at their actual decision points.
- [x] **M1.6 Refresh project discovery:** add the child route to the `zircon-project-skills` parent, regenerate or update the cached project skill catalog from the shallow tree, and confirm `agents/openai.yaml` quotes all strings and names `$close-session-goal-milestones` in `default_prompt`.
- [x] **M1.7 Forward-test the deployed skill:** give a fresh agent the skill and the same pressure scenario with raw fixture facts, without revealing the expected solution. Confirm it separates file categories, refuses foreign scope and Session tags, commits at an accepted milestone, keeps Session active for Milestone mode, completes only in Goal mode, and reports a failed WeCom send without retrying.

### Lightweight checks

- Run `git diff --check` on the new skill and catalog paths after each substantial edit.
- Parse the PowerShell scripts before the testing stage with `[System.Management.Automation.Language.Parser]::ParseFile(...)`.
- Do not run tests against the real repository; all checker fixtures use temporary repositories.

### Testing stage M1-T: Skill and checker acceptance

- [x] Run the checker suite:

  ```powershell
  Import-Module Pester -ErrorAction Stop
  $result = Invoke-Pester -Script .\.codex\skills\zircon-project-skills\close-session-goal-milestones\scripts\check-closeout.Tests.ps1 -PassThru
  if ($result.FailedCount -gt 0) { exit 1 }
  ```

  Expected: every positive and negative closeout case passes with zero failures.

- [x] Run both skill validators:

  ```powershell
  python -X utf8 "$HOME\.codex\skills\.system\skill-creator\scripts\quick_validate.py" .codex\skills\zircon-project-skills\close-session-goal-milestones
  .\.codex\skills\project-skills-index\scripts\list-skill-tree.ps1
  ```

  Expected: `Skill is valid!`; the shallow tree contains `close-session-goal-milestones/`.

- [x] Run catalog, placeholder, scope, and secret checks:

  ```powershell
  git diff --check -- .codex/skills/zircon-project-skills .codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md
  git diff -- .codex/skills/zircon-project-skills .codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md
  ```

  Expected: no whitespace errors, no generated placeholder markers, no literal webhook URL/key, and no unrelated staged path.

- [x] Debug/correction loop: on any checker, parser, quick-validation, forward-test, or review failure, fix the lowest shared skill/checker rule and rerun the focused failing case before repeating M1-T upward.
- [x] Request independent review of trigger quality, Milestone/Goal distinction, read-only guarantees, shared-main isolation, credential handling, catalog consistency, and test coverage. Require no Critical or Important findings.
- [ ] Commit exactly the skill, checker, tests, parent route, catalog, and this plan's final navigation status. Use a normal subject such as `feat(workflow): add Session goal milestone closeout skill`; never use `[zircon-session:*]`.
- [ ] Immediately push the successful commit to WeCom in exactly four newline-separated lines and do not retry automatically if sending fails.

### Exit evidence

- Pester closeout suite passes with zero failures.
- `quick_validate.py` reports the skill valid.
- The refreshed shallow skill tree and catalog contain the new child skill.
- Forward-test follows both Milestone and Goal paths without foreign-file or notification-policy violations.
- Scoped diff/secret checks pass and independent review has no Critical/Important findings.
- The implementation commit exists on `main`, contains only the explicit skill scope, and its WeCom result is recorded.

## Milestone Promotion Rules

- Do not mark M1 complete until M1-T passes and the implementation commit succeeds.
- Do not treat a successful skill validator as proof that the PowerShell checker is correct; both Pester and forward-testing are required.
- Do not run the checker against the real dirty repository during tests.
- If another Session owns a target file, keep the skill work active, enqueue a delayed patch or wait for the lease, and do not overwrite.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。具体证据写入本计划导航表；本计划位于 `docs/superpowers/plans`，不写入业务 `docs/plans` 全局索引。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | M1.1 RED pressure evidence | 完成 | 2026-07-11 | Fresh no-skill scenario omitted the milestone's untracked test, accepted `[zircon-session:*]`, completed a nonterminal Session, and retried failed WeCom delivery. |
| M1 | M1.2 Skill scaffold | 完成 | 2026-07-11 | `init_skill.py` created `close-session-goal-milestones/` with `SKILL.md`, quoted `agents/openai.yaml`, and an empty `scripts/` resource directory. |
| M1 | M1.3 Checker tests | 完成 | 2026-07-11 | `check-closeout.Tests.ps1` covers 26 Milestone/Goal, real-service health, lease, deletion, staged/worktree divergence, current-vs-historical child-plan scope, untracked, branch, message, credential, related/foreign Failure diagnostics, and aggregate-completion cases; initial RED run failed 0/9 because the checker did not exist. |
| M1 | M1.4 Read-only checker | 完成 | 2026-07-11 | `check-closeout.ps1` validates manifest categories, exact staged scope, live coordinator health/lease/current-hash attribution, current staged numbered-child plan evidence, related canonical Failure state, Conventional Commit policy, staged secrets, and mode semantics without mutation; Pester 26/26 and Python reader 1/1 passed. |
| M1 | M1.5 Skill workflow | 完成 | 2026-07-11 | `SKILL.md` defines real-service preflight, pre-write leases, immediate atomic milestone commit, terminal Goal completion, five category inventory, exact scope, no Session tag, and single-attempt four-line WeCom handling; its 670-word size retains the concrete service command and both closeout modes. |
| M1 | M1.6 Discovery refresh | 完成 | 2026-07-11 | Parent `zircon-project-skills/SKILL.md`, cached project catalog, and quoted `agents/openai.yaml` now route and describe `close-session-goal-milestones`. |
| M1 | M1.7 Forward test | 完成 | 2026-07-11 | Fresh skill-guided scenario included the omitted untracked test in both categories, stopped on foreign staged scope, rejected Session tags, kept Milestone Session/Goal active, and refused automatic WeCom retry. |
| M1 | M1-T Acceptance | 完成（待本次提交） | 2026-07-11 | Pester 26/26, coordinator Python suite 121/121 plus focused Failure/finalize additions, read-only coordinator evidence test 1/1, skill validator, scoped static checks, and two independent no-Critical/no-Important reviews pass. |
