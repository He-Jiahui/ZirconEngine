# Code Module Docs Maintenance

## Overview

Use this skill only when durable documentation is necessary to keep a public/cross-module fact true. Source and tests are the default facts; concise comments explain non-obvious invariants, and the numbered plan records completion state. Documentation must not become a second, drifting implementation.

## Progressive Disclosure Index

- Start with `../milestone-first-workflow-policy.md` for the required comment, documentation, and milestone testing cadence.
- Start with `plan-docs-directory-first.md`.
- If you need the required machine-readable document header, read `required-doc-header-format.md`.
- If you need the full writing rules for module documentation, read `write-module-docs/index.md`.
- If you need the synchronization rules for code changes, read `maintain-docs-with-code-changes/index.md`.
- Also apply `../layered-milestone-development/guide.md` when the documentation is tied to a milestone or subsystem plan.
- Use the existing validation evidence when documenting acceptance. Apply `../evidence-driven-wsl-validation/guide.md` only for a concrete Linux-specific validation requirement.

## Non-Negotiable Rules

- Do not create or expand `docs/` for ordinary slices, private refactors, test-only changes, or facts already clear in source and tests.
- Do not create a document merely to report work performed, a command result, a review, or a plan update. Those belong in source/tests, relevant retained validation evidence, or the single milestone status row.
- Update an existing document in the same task only when it would otherwise make a materially false claim about a public contract, cross-module boundary, operator workflow, or durable design decision.
- Create a source-mirrored document only when a new public/cross-module interface or operational workflow has no concise existing owner. Prefer an existing functional document when it owns the behavior.
- Keep retained documents compact and factual: the owner modules, the changed contract/decision, and the relevant validation suite or plan milestone. Remove stale text; never append a per-slice changelog.
- Keep a machine-readable header only on retained documents, and list the owner modules/test suites rather than every touched implementation file.
- Put concise comments in code for non-obvious invariants and transitions; do not use documentation as a substitute for code clarity.
