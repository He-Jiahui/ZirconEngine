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

For a plan executed in a separate Session, also read `../../../zircon-project-skills/cross-session-coordination/references/model-tier-policy.md`. Every delegated task must declare an allowed `5.6-sol`, `5.6-terra`, or `5.6-luna` tier and Light/Medium/High/Extra High thinking depth. Do not use or silently fall back to `gpt-5.5` or lower.

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
- Follow the repository's configured integration workflow. In ZirconEngine, stay on `main` and use the coordinator-managed closeout skill when an accepted closeout is due.

## When to Stop and Ask for Help

**Reprioritize before asking for help:**
- For a ZirconEngine blocker, apply `continuous-milestone-execution`: resolve an applicable Failure first, then continue independent Goal work.
- Ask only when no independent in-scope work remains and a real product decision or missing requirement prevents progress.

## When to Revisit Earlier Steps

**Return to Review (Step 1) when:**
- Partner updates the plan based on your feedback
- Fundamental approach needs rethinking

Do not force through a product ambiguity; otherwise continue independent work while the dependent slice is unavailable.

## Remember
- Review plan critically first
- Follow plan steps exactly
- Don't skip verifications
- Reference skills when plan says to
- Stop when blocked, don't guess
- Honor `zirconEngine` milestone-first testing cadence over generic per-task verification loops
- Honor repository-local git workflow policy before assuming worktree or feature-branch isolation

## Integration

**Related workflow skills:**
- **superpowers:subagent-driven-development** - Use for independently delegable implementation-plan tasks.
