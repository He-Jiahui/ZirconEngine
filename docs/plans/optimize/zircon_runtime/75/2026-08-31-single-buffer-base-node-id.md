# Runtime75 Single-Buffer Palette Base Node ID

- Date: 2026-08-31
- Parent plan: `docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md`
- Status: implementation_complete; managed_validation_pending
- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`

## Problem

Every palette node creation normalized its label by collecting one mapped `String`, borrowing a
trimmed slice from it, and then allocating a second lowercase `String`. The first buffer was dropped
immediately after the final identifier was produced.

## Optimization

- Preallocate one buffer to the label's UTF-8 byte length.
- Normalize and lowercase ASCII alphanumeric characters in one traversal.
- Skip leading separators, retain interior separators exactly, remember the last non-separator byte
  boundary, and truncate the trailing separators once.
- Preserve the `"node"` fallback for empty normalized identifiers.

## TDD And Verification

- RED: the new source contract failed against the collect/trim/lowercase chain.
- GREEN: current palette normalization, mount-set, and native-slot contracts pass `5/5`.
- Rust behavior coverage compares empty, punctuation-only, whitespace, repeated separators,
  uppercase ASCII, combining marks, and non-ASCII boundary inputs with the legacy contract.
- Python bytecode compilation and scoped `git diff --check` pass.

## Performance Gate

The ignored release benchmark performs 100,000 normalizations per sample over 21 alternating
legacy/optimized pairs, emits raw series and nearest-rank P50/P95, and requires optimized P95 to be
at least 20% lower.

Deterministic allocation count per non-empty identifier:

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| normalization `String` allocations | 2 | 1 | -50.000% |
| temporary full-label buffers | 1 | 0 | -100% |

Measured timings remain pending the managed Windows release batch.

## Remaining Scope

This change is limited to palette node identity normalization. Component catalog authority,
palette generation caching, schema admission, and broader Editor workflows remain open.
