---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/709-accessibility-resolution-key-scratch-reuse.md
  - docs/plans/astra/features/runtime/711-accessibility-filter-map-membership.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
---

# Accessibility Description Reference Borrowing

Description resolution only needs to mutate entries whose description starts with the typed
`#<node-id>` reference form. The resolver now probes `description` through `as_deref()` and
returns immediately for ordinary final text, materializing an owned copy only for a reference
that may be replaced or cleared. Malformed and dangling references continue through the same
diagnostic and removal paths, so ordering and failure semantics are unchanged.

This removes a transient `String` clone for every ordinary description in an accessibility
snapshot while retaining one owned value where replacement accounting requires it. The change is
local to the resolver and does not alter the public accessibility DTO or the existing key-scratch
reuse from record 709.

## Evidence

- Runtime accessibility extraction and rich-text source contracts passed `24/24`, including the
  focused accessibility extraction contract `4/4`.
- Scoped Rustfmt and `git diff --check` passed for the changed resolver.
- Current resolver SHA-256:
  `48B9667E86E153B9E27FA91A301A268F7F67D426F8FA845C081C7E411126FB0F`.

These are source/static and deterministic-contract results. Managed Cargo, Windows Release
allocation/time, and product accessibility latency evidence remain pending under the external
dirty-worktree admission gate; no product performance acceptance is claimed here.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/78 | Borrow-probe ordinary descriptions and clone only reference candidates during accessibility resolution | implemented_pending_validation | Focused accessibility/rich-text contracts `24/24`, Rustfmt, diff, and source-hash guards pass; managed Rust and product measurements remain pending |
