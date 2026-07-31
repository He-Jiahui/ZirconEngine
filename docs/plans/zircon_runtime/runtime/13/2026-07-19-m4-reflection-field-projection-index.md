---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --no-default-features --features script --locked --jobs 1 reflect_registration_ -- --nocapture --test-threads=1
doc_type: milestone-detail
---

# Runtime13 M4 Reflection Field Projection Index

Plan: `docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`

Milestone: M4 current architecture and performance convergence

Status: `implementation_review_corrections_applied_revalidation_pending`

Files: `zircon_runtime/src/core/framework/script.rs`, `zircon_runtime/src/script/vm/tests/reflection_docs.rs`

Date: 2026-07-19

## Implementation

Reflection registration now builds one name-to-kind index from the projected fields before walking the reflected fields. Projection validation and output construction therefore run in `O(F + P)` instead of repeatedly scanning the reflected field list for every projected field.

The hard contract remains explicit:

- duplicate projected field names produce the existing typed invalid-registration error before lookup;
- unknown projected names are reported before missing reflected fields;
- missing reflected fields preserve reflected declaration order;
- successful output preserves reflected declaration order, matched projection value kinds, and reflected type references rather than projection input order;
- one-field, 100-field, and 10,000-field reverse-order projections with distinct per-field type references exercise the same indexed owner;
- combined unknown-plus-multiple-missing coverage locks unknown-before-missing precedence and the first missing reflected declaration.

No compatibility projection path, fallback linear scan, duplicate descriptor owner, or relaxed error ordering was added.

## Validation and review

- Scoped rustfmt, source guards, and `git diff --check` passed.
- Correction re-review reported Critical 0 / Important 0 / Minor 1. The remaining record-only wording correction is applied in the current candidate; fresh source-bound validation and final re-review are pending.
- Superseded reservation `3f6938d48b224ef492bcece3503170c0` was released before these edits. Its old validation copy/evidence must not be cited for the corrected source.

## Boundary

This record does not promote the parent Runtime13 plan to complete. The managed focused gate, final source-bound review, and exact milestone commit remain required.
