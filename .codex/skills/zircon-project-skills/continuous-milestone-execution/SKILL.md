---
name: continuous-milestone-execution
description: Execute `zirconEngine` work continuously against an existing milestone or plan instead of stopping after partial analysis or partial implementation. Use when Codex must fully read the current workspace crates, CI constraints, and relevant plan artifacts first, then keep generating, validating, and advancing the current milestone until it is actually complete, only pausing when a real branch ambiguity requires a user decision.
---

# Continuous Milestone Execution

## Overview

Use this skill to enforce uninterrupted, plan-driven `zirconEngine` execution once the current milestone path is known.

## Progressive Disclosure Index

- Start by reading `../milestone-first-workflow-policy.md`; it defines the cadence for implementation slices, testing stages, and documentation.
- Start with `read-code-and-plans-before-acting.md`.
- If you need the execution loop that keeps work moving until the milestone is done, read `execution-loop/SKILL.md`.
- If you need to decide whether asking the user is allowed, read `stop-only-for-real-branch-ambiguity.md`.
- If the milestone itself is still unclear or not yet structured, also apply `../layered-milestone-development/SKILL.md`.
- If execution fails because an upper layer depends on a broken lower layer, also apply `../support-first-regression-testing/SKILL.md`.
- If the lowest broken owner belongs to another numbered plan, apply `../handle-plan-failure-handoffs/SKILL.md` instead of stopping the whole source session.

## Non-Negotiable Rules

- Read the current code, tests, and relevant plan files before deciding what to build next.
- **Failure Priority Gate:** scan the active numbered child-plan directory for `failure-*.md` before normal feature slices. When this Session owns the fixing plan, enter `resolving_failure` and complete the architectural repair, upward validation, and `failure return` before any normal slice. An origin Session may continue only dependency-independent slices.
- Continue implementation and validation until the active milestone is complete.
- During implementation slices, do not force immediate build/unit-test loops. Write tests and docs as needed, but defer compile and unit-test execution to the milestone testing stage unless the user asks otherwise or a blocker requires evidence.
- After each completed slice, immediately update the active plan's required status/output table with one evidence row; do not batch-fill progress records later.
- Do not stop after partial code generation, partial debugging, or partial test work just because a local subtask ended.
- Ask the user only when multiple materially different branches remain valid and the choice is product-level, not merely technical effort.
- Treat "need more analysis" as a reason to keep reading and executing, not as a reason to pause the work.
- Treat a cross-plan failure as a handoff plus reprioritization event: publish it to the fixing child plan, continue independent source-plan slices, and never use it alone to mark the session blocked.
