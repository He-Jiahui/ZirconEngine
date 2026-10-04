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
  - docs/plans/astra/features/runtime/710-accessibility-description-reference-borrow.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
---

# Accessibility Child Filtering Uses the Published Map

The accessibility child-filter pass already owns the authoritative published-node
`BTreeMap`, but it previously cloned every key into a temporary `BTreeSet` before
recursively checking membership. The resolver now passes the map directly to the
recursive collector and uses `contains_key`, while retaining the separate ordered
key snapshot needed for value mutation.

This removes one tree allocation and repeated ordered inserts per accessibility
snapshot. Child flattening order, hidden-target suppression, budget accounting,
and missing-node behavior remain unchanged.

## Evidence

- `child_filter_uses_authoritative_map_membership_without_key_set_clone` guards
  the direct map membership path and the retained ordered key snapshot.
- Focused accessibility/rich-text contracts passed `24/24` in `0.034s`; Runtime
  and Editor performance suites passed `1146/1146` in `5.703s` and `581/581`
  in `1.236s`; full pressure suites passed `154/154` in `3.897s` and `129/129`
  in `3.365s` after this change.
- Rustfmt and scoped `git diff --check` passed; the Runtime711 source guard
  passed `3/3`.
- The deduplicated Runtime/Editor performance, pressure, accessibility,
  rich-text, and Editor607-focused loader passed `2041/2041` in `11.943s` on
  the resulting source snapshot.

These are source and deterministic contract results. Managed Cargo, Windows
Release allocation/time, and product accessibility latency evidence remain
pending under the external dirty-worktree admission gate.

Current `resolution.rs` SHA-256:
`48B9667E86E153B9E27FA91A301A268F7F67D426F8FA845C081C7E411126FB0F`.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/78 | Use the published accessibility map for child membership instead of cloning a key `BTreeSet` | implemented_pending_validation | Focused accessibility/rich-text contracts, source guard, Rustfmt, and scoped diff checks pass; managed Rust and product measurements remain pending |
