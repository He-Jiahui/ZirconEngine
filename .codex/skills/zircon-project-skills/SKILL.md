---
name: zircon-project-skills
description: Use when Codex needs repository-local `zirconEngine`-specific guidance that should be routed through categorized child skills under `.codex/skills/zircon-project-skills`, especially for architecture, module structure, execution discipline, coordination, validation, or repository-specific maintenance work.
---

# Zircon Project Skills

## Start Here

- Read this parent file first instead of scanning every child skill under `zircon-project-skills`.
- Read `../zircon-engineering/SKILL.md` first for the default delivery loop and capability tier.
- Read `milestone-first-workflow-policy.md` when planning a milestone, selecting a validation gate, or writing plan evidence.
- Use `tools/zircon-session.ps1` only when another active Session may overlap the same files, plan, or failure.
- Load `architecture-overview.md` and `development-conventions.md` only for C3 architecture, public-boundary, or hard-migration work.
- Apply `handle-plan-failure-handoffs/SKILL.md` only when a discovered failure applies to the active milestone or its direct dependency.
- Choose the smallest child skill group that matches the task, then open only the referenced child `SKILL.md`.
- If the task fits `zircon-dev`, use `../zircon-dev/SKILL.md` only when Rust workspace editing or a Cargo gate is actually involved.

## Progressive Disclosure Index

- For architecture shape, module boundaries, reference-engine selection, or constant-placement discipline:
  - `zr-architecture-first-engineering/SKILL.md`
  - `zr-hard-cutover-migrations/SKILL.md`
  - `zr-runtime-interface-convergence/SKILL.md`
  - `zr-language-feature-design/SKILL.md`
  - `zr-reference-engine-routing/SKILL.md`
  - `zr-module-boundary-discipline/SKILL.md`
  - `zr-magic-constant-convergence/SKILL.md`
- For execution order, validation depth, regression repair, or continuous milestone delivery:
  - `layered-milestone-development/SKILL.md`
  - `continuous-milestone-execution/SKILL.md`
  - `prefer-windows-validation/SKILL.md`
  - `evidence-driven-wsl-validation/SKILL.md`
  - `support-first-regression-testing/SKILL.md`
- For Hub visual verification and real Windows window screenshots:
  - `capture-hub-window-screenshot/SKILL.md`
- For repository coordination, minimal plan/failure record ownership, or a durable-documentation exception:
  - `cross-session-coordination/SKILL.md`
  - `close-session-goal-milestones/SKILL.md`
  - `handle-plan-failure-handoffs/SKILL.md`
  - `write-plan-output-records/SKILL.md`
  - `code-module-docs-maintenance/SKILL.md`
  - `modularize-large-files/SKILL.md`

## Rules

- Keep this parent file short and navigational.
- Treat this folder as a classified skill collection, not as a flat dump of unrelated repository skills.
- Treat `milestone-first-workflow-policy.md` as the repository-level execution cadence for plans, implementation slices, validation, and docs.
- Do not route ordinary source work into documentation maintenance: source, focused tests, coordinator state, and one accepted milestone row are the default record; load `code-module-docs-maintenance` only when a durable fact would otherwise be false or absent.
- Treat coordinator SQLite state as a local query/index layer only. Git history and canonical Markdown plans/failure artifacts remain repository truth.
- Add new child skills only when their scope is specific enough to deserve a dedicated folder and `SKILL.md`.
- Update this parent index whenever a child skill is added, renamed, split, merged, or materially repurposed.
- If the child list becomes hard to scan from a shallow listing, split it into category subfolders with their own parent `SKILL.md` files instead of letting the root index bloat.
