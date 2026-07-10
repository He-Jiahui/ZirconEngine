---
name: executing-plans
description: Use when you have a written implementation plan to execute in a separate session with review checkpoints
---

# Executing Plans

## Overview

Load plan, review critically, execute all tasks, report when complete.

**Announce at start:** "I'm using the executing-plans skill to implement this plan."

**Note:** Tell your human partner that Superpowers works much better with access to subagents. The quality of its work will be significantly higher if run on a platform with subagent support (such as Claude Code or Codex). If subagents are available, use superpowers:subagent-driven-development instead of this skill.

**Repository-local workflow policy overrides this skill.** In `zirconEngine`, read `../../../zircon-dev/references/main-branch-development-policy.md` before starting and keep all work on `main`.

In `zirconEngine`, also read `../../../zircon-project-skills/milestone-first-workflow-policy.md`. Do not turn each plan task into its own compile/build/unit-test loop when the plan separates implementation slices from a milestone testing stage.

## The Process

### Step 1: Load and Review Plan
1. Read plan file
2. Review critically - identify any questions or concerns about the plan
3. If concerns: Raise them with your human partner before starting
4. If no concerns: Create TodoWrite and proceed

### Step 2: Execute Tasks

For each task:
1. Mark as in_progress
2. Follow each step exactly (plan has bite-sized steps)
3. Run verifications as specified, except that `zirconEngine` milestone plans defer compile/build/unit-test commands to the named testing stage unless the user explicitly requested earlier validation
4. Mark as completed

### Step 3: Complete Development

After all tasks complete and verified:
- If the repository has a local main-branch-only policy, stay on `main`, summarize the verification evidence, and report completion without branch or worktree closeout steps.
- If the repository does not have that policy, announce: "I'm using the finishing-a-development-branch skill to complete this work."
- Then use **REQUIRED SUB-SKILL:** superpowers:finishing-a-development-branch
- Then follow that skill to verify tests, present options, and execute the chosen integration path.

## When to Stop and Ask for Help

**STOP executing immediately when:**
- Hit a blocker (missing dependency, test fails, instruction unclear)
- Plan has critical gaps preventing starting
- You don't understand an instruction
- Verification fails repeatedly

**Ask for clarification rather than guessing.**

## When to Revisit Earlier Steps

**Return to Review (Step 1) when:**
- Partner updates the plan based on your feedback
- Fundamental approach needs rethinking

**Don't force through blockers** - stop and ask.

## Remember
- Review plan critically first
- Follow plan steps exactly
- Don't skip verifications
- Reference skills when plan says to
- Stop when blocked, don't guess
- Honor `zirconEngine` milestone-first testing cadence over generic per-task verification loops
- Honor repository-local git workflow policy before assuming worktree or feature-branch isolation

## Integration

**Required workflow skills:**
- **superpowers:using-git-worktrees** - Use only when the repository policy allows isolated worktrees
- **superpowers:writing-plans** - Creates the plan this skill executes
- **superpowers:finishing-a-development-branch** - Use only when the repository is actually following a branch-based integration flow
