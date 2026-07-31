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
- Scan for applicable `failure-*.md` at milestone start and when related validation fails. A failure blocks dependent slices, while origin work may continue on independent slices.
- Continue implementation and validation until the active milestone is complete.
- During implementation slices, do not force immediate build/unit-test loops. Produce code and focused tests first; write prose only for a durable public/cross-module fact that would otherwise be undocumented or false. Defer compile and unit-test execution to the milestone testing stage unless the user asks otherwise or a blocker requires evidence.
- Do not create per-slice plan output rows, test transcripts, or narrative closeouts. Record one concise accepted milestone outcome after its validation gate.
- Do not stop after partial code generation, partial debugging, or partial test work just because a local subtask ended.
- Ask the user only when multiple materially different branches remain valid and the choice is product-level, not merely technical effort.
- Treat "need more analysis" as a reason to keep reading and executing, not as a reason to pause the work.
- Treat a cross-plan failure as a handoff plus reprioritization event: publish it to the fixing child plan, continue independent source-plan slices, and never use it alone to mark the session blocked.
- Treat an applicable coordinator-generated `failure-*.md` owned by the current Plan as the first executable item. Switch to `resolving_failure` and repair it before ordinary slices; do not defer it behind a later feature or a pending validation ticket.

## Receipt-Driven Validation And Integration

- After the coordinator accepts a validation or integration request and returns a durable ticket or queued receipt, treat the snapshot as having no known validation failure and immediately rebuild the Goal action queue. Do not poll, repeat the request, or hold a Session turn waiting for validation.
- Report a coordinator-integrated snapshot as `integrated_validation_pending`, never `accepted`, until the coordinator supplies complete validation evidence. Preserve its actual `main` commit and assume forward progress unless a canonical `failure-*.md` arrives; do not manually revert an integrated snapshot for an ordinary test failure.
- When the coordinator has not yet returned an integration SHA, keep the candidate owned and queued. Do not use direct Git actions or a claimed `accepted` status to simulate integration; continue independent implementation, review, static checks, and repair work instead.
- When all planned implementation milestones have been submitted or integrated, perform a second review of the delivered in-scope code, ownership evidence, known Failure records, and outstanding validation tickets. Repair findings immediately; otherwise release the turn for coordinator wakeup rather than waiting or polling.

## When a Slice Is Blocked

Keep the blocked slice visible, then immediately select the highest-priority actionable work from the same Goal:

| Priority | Required action |
| --- | --- |
| 1 | Resolve an applicable current-Plan `failure-*.md`, establish the lowest owned cause, and return the repair. |
| 2 | Implement another dependency-independent slice in the active milestone. |
| 3 | Implement an approved downstream Goal slice or milestone when its entry criteria are satisfied and it shares neither the blocked dependency nor a conflicting write scope. |
| 4 | Review completed in-scope code and address safe review findings. |
| 5 | Inspect in-scope validation failures that do not yet have a Failure record, then repair or locally validate the lowest owned cause. |

- Treat a coordinator outage, lease conflict, unavailable CI worker, or cross-plan handoff as blocking only its dependent work, not the entire Goal.
- Rebuild this actionable queue after each completed item and whenever a blocker changes; return to the blocked slice as soon as it becomes actionable.
- Mark the Session or Goal `blocked` only after re-scanning the Goal shows that every remaining in-scope item requires the same unresolved external dependency or a genuine user product decision.
- Do not use strict milestone order, waiting for a coordinator response, or a pending CI run as a reason to idle when an approved independent Goal item remains.
