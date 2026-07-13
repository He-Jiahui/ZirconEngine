---
name: zircon-project-skills
description: Use when Codex needs repository-local `zirconEngine`-specific guidance that should be routed through categorized child skills under `.codex/skills/zircon-project-skills`, especially for architecture, module structure, execution discipline, coordination, validation, or repository-specific maintenance work.
---

# Zircon Project Skills

## Start Here

- Read this parent file first instead of scanning every child skill under `zircon-project-skills`.
- Read `milestone-first-workflow-policy.md` before planning, implementing, validating, or documenting `zirconEngine` work. It overrides generic workflows that would force per-slice compile/test loops.
- Use `tools/zircon-session.ps1` as the local coordination entrypoint for Session enum state, numbered-plan ownership, Failure graph priority, file leases, delayed patches, baselines, and snapshots.
- Read `architecture-overview.md` and `development-conventions.md` before starting to plan and develop.
- Before advancing a numbered child plan, scan its `{id}/` directory for `failure-*.md`; if any exist, apply `handle-plan-failure-handoffs/SKILL.md` before normal feature work.
- Choose the smallest child skill group that matches the task, then open only the referenced child `SKILL.md`.
- If the task already fits `zircon-dev`, use `../zircon-dev/SKILL.md` for workspace workflow and validation, then come back here only for `zirconEngine`-specific project disciplines.

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
- For repository coordination, documentation synchronization, or maintenance-focused structure work:
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
- Treat coordinator SQLite state as a local query/index layer only. Git history and canonical Markdown plans/failure artifacts remain repository truth.
- Add new child skills only when their scope is specific enough to deserve a dedicated folder and `SKILL.md`.
- Update this parent index whenever a child skill is added, renamed, split, merged, or materially repurposed.
- If the child list becomes hard to scan from a shallow listing, split it into category subfolders with their own parent `SKILL.md` files instead of letting the root index bloat.
