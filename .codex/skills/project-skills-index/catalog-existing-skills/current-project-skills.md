# Current Project Skills

Updated: 2026-07-11

## Shallow Tree

```text
.codex/skills/
  project-skills-index/
    agents/
    catalog-existing-skills/
    scaffold-indexed-skill/
    scripts/
    enumerate-existing-skills.md
    generate-indexed-skill-layout.md
    SKILL.md
  superpowers/
    foundations-and-planning/
    implementation-and-delegation/
    review-and-stabilization/
    skill-authoring/
    scripts/
    brainstorm.md
    code-reviewer.md
    execute-plan.md
    SKILL.md
    survey-top-level-layout.md
    write-plan.md
  zircon-dev/
    agents/
    references/
    scripts/
    validation/
    workflow/
    reporting.md
    SKILL.md
  zircon-project-skills/
    SKILL.md
    capture-hub-window-screenshot/
    code-module-docs-maintenance/
    continuous-milestone-execution/
    cross-session-coordination/
    evidence-driven-wsl-validation/
    handle-plan-failure-handoffs/
    layered-milestone-development/
    modularize-large-files/
    support-first-regression-testing/
    zr-architecture-first-engineering/
    zr-hard-cutover-migrations/
    zr-language-feature-design/
    zr-magic-constant-convergence/
    zr-module-boundary-discipline/
    zr-reference-engine-routing/
    zr-runtime-interface-convergence/
    zr-workspace-structure-hard-cutover/
```

## Summaries

- `project-skills-index`
  Summary: Catalog and scaffold repository-local Codex skills with progressive disclosure. Use at the start of work in this repo when Codex needs a shallow inventory of `.codex/skills`, needs to capture or refresh each skill's top-level summary, or needs to create or update a local skill using a parent-skill plus child-skill chunked layout.
  Layout: Indexed parent skill with short root instruction files, child `*/SKILL.md` guides, and shallow tree scripts for PowerShell and WSL.

- `superpowers`
  Summary: Navigate and choose the right superpowers development workflow skill with progressive disclosure. Use when Codex needs to find, triage, or maintain the curated superpowers skill set for planning, implementation, delegation, debugging, review, verification, and skill authoring workflows.
  Layout: Physically categorized skill tree with four workflow branches, each branch using a short category `SKILL.md` plus the actual leaf skills beneath it.

- `zircon-dev`
  Summary: Use when building, testing, refactoring, or debugging the `zirconEngine` Rust workspace, especially for Rust code, Cargo manifests, workspace wiring, or cross-crate boundaries that touch the `zircon_app`/`zircon_runtime`/`zircon_editor` root packages or the runtime-internal `core/{runtime,framework,manager,math,resource}` spine.
  Layout: Structured repository skill with a short root `SKILL.md`, focused child folders for `workflow/` and `validation/`, a dedicated `reporting.md`, plus reusable `references/`, `scripts/`, and `agents/` metadata.

- `zircon-project-skills`
  Summary: Route repository-local `zirconEngine` skills through a parent category index instead of scanning the whole project-specific skill collection.
  Layout: Parent-indexed repository skill collection with a short root `SKILL.md` that groups child skills by architecture, execution/validation, Hub visual checks, and coordination/maintenance concerns, while the focused leaf skills remain in their dedicated child folders.

- `capture-hub-window-screenshot`
  Summary: Capture the actual Windows Zircon Hub Slint window for visual comparison, including robust top-level window selection and topmost screen capture.
  Layout: Focused child skill under `zircon-project-skills` with a short `SKILL.md`, `agents/openai.yaml`, and reusable PowerShell script in `scripts/capture-hub-window.ps1`.

- `handle-plan-failure-handoffs`
  Summary: Enforce durable cross-plan failure/fixed handoffs when a ZirconEngine failure belongs to another numbered child plan or a verified repair must return to its originating executor.
  Layout: Focused child skill with a concise workflow, reusable Markdown schema, and a standard-library validator plus positive/negative fixture tests.

- `zr-module-boundary-discipline`
  Summary: Enforce strict single-responsibility source boundaries in `zirconEngine`. Use when Rust modules or directories start mixing multiple feature domains, root wiring files gather logic, or similar files need a folder-backed subtree instead of flat growth.
  Layout: Focused skill with a strict root `SKILL.md` plus `references/binding-rs-anti-pattern.md` showing the `zircon_editor_ui/src/binding.rs` anti-pattern and one acceptable decomposition shape.

## Maintenance Rule

- Treat this file as a cached index.
- Rebuild it from the current filesystem and top-level frontmatter when anything under `.codex/skills` changes.
