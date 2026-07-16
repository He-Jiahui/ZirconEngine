---
name: layered-milestone-development
description: Use when ZirconEngine work needs dependency-ordered milestones for shared runtime, module wiring, scripting, editor, graphics, asset, platform, or other cross-crate changes.
---

# Layered Milestone Development

## Overview

Use this skill to keep `zirconEngine` development aligned with repository structure, lower-to-upper dependency order, and hard milestone acceptance criteria.

## Progressive Disclosure Index

- Start by reading `../milestone-first-workflow-policy.md`; it defines milestone testing stages and the no per-slice build/test cadence.
- Start with `analyze-current-project-structure-first.md`.
- If you need to split work into dependency-ordered milestones, read `plan-by-layer/SKILL.md`.
- If you need to decide whether a milestone is allowed to advance, read `milestone-completion-requires-full-coverage.md`.
- If you need the full validation workflow and reporting rules, read `validate-before-promoting/SKILL.md`.
- If an upper-layer feature fails, also apply `../support-first-regression-testing/SKILL.md`.

## Non-Negotiable Rules

- Analyze the current repository structure, goals, and existing baseline before writing or executing a milestone plan.
- Develop from lower shared layers to higher orchestration layers.
- Every plan must include a named testing stage per milestone. Implementation slices may write unit-test code, but compile/build/unit-test execution belongs to the testing stage unless explicitly requested earlier.
- Every milestone plan must include the required status/output record section; record one accepted milestone outcome after its testing stage, not a row for each slice.
- Do not mark a milestone complete from smoke tests, empty tests, or one happy-path example.
- Treat untested changed contracts and high-risk boundaries as incomplete work; unchanged equivalent forms reuse existing coverage per `milestone-completion-requires-full-coverage.md`.
- If an upper-layer milestone fails, re-check the supporting lower layers instead of inventing a special upper-layer-only success path.
