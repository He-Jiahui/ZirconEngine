---
name: code-module-docs-maintenance
description: Generate and maintain detailed `zirconEngine` module documentation whenever code is created, changed, or reorganized. Use when shared runtime, module/plugin wiring, scripting, editor/graphics, asset, networking, or other cross-crate work must also emit or update detailed docs under `docs/`, organize docs by subsystem, include machine-readable related-code headers for script lookup, and record plan sources, tests, and implementation files in every affected document.
---

# Code Module Docs Maintenance

## Overview

Use this skill to make code generation and code documentation move together in `zirconEngine`, with subsystem-oriented `docs/` categorization, machine-readable headers, and mandatory maintenance whenever code changes.

## Progressive Disclosure Index

- Start with `../milestone-first-workflow-policy.md` for the required comment, documentation, and milestone testing cadence.
- Start with `plan-docs-directory-first.md`.
- If you need the required machine-readable document header, read `required-doc-header-format.md`.
- If you need the full writing rules for module documentation, read `write-module-docs/SKILL.md`.
- If you need the synchronization rules for code changes, read `maintain-docs-with-code-changes/SKILL.md`.
- Also apply `../layered-milestone-development/SKILL.md` when the documentation is tied to a milestone or subsystem plan.
- Also apply `../evidence-driven-wsl-validation/SKILL.md` when the documentation must record validation evidence and acceptance coverage.

## Non-Negotiable Rules

- When you generate or materially change code, also generate or update the corresponding docs under `docs/` before calling the work complete.
- Use `docs/` paths that mirror the source module path when creating new module-level documents, unless an existing functional document already owns that module.
- Organize higher-level docs by functional areas, not by random dates or vague buckets.
- Put all related code files at the top of every documentation file in a machine-readable header so scripts can map code files back to docs.
- Document plan source, test coverage, and implementation files in every affected document.
- Keep docs detailed. Do not reduce them to changelog fragments or short summaries when design and behavior need explanation.
- If code changes invalidate a document, update that document in the same task. Do not defer doc maintenance silently.
- Key data structures and non-obvious logic in the code must have concise comments before the docs call the module understandable.
