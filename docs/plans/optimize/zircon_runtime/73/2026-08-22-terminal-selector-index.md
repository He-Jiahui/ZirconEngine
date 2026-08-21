# Runtime73 Terminal Selector Index

Plan: docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
Milestone: M3
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/73/2026-08-22-terminal-selector-index.md","zircon_runtime/src/ui/v2/style.rs","zircon_runtime/src/ui/v2/style/rule_index.rs"]

- Date: 2026-08-22
- Integration owner: `optimize-runtime73-runtime81-runtime89-batch-m3-r1-01a00797-20260822`
- Former owner: `optimize-runtime73-terminal-selector-index-m3-r2-01a00797-20260822`
  (`cancelled` after grouped transfer fingerprint
  `2b9dfa137351b0771e7c7f4fa03b571010756adbbf0c8a60bde3ca394807aa29`)
- Superseded scope owner: `optimize-runtime73-terminal-selector-index-m3-r1-01a00797-20260822`
  (`cancelled` to establish the immutable module-extraction scope; transfer fingerprint
  `a9e21f8ca574b272ca98db185981925fe3ccea05d1d5618ef4e31e8757a45f2f`)
- Inherited Runtime09 change: pseudo-state dependency names transferred through fingerprint
  `a66ee071a69ab38a1fe114f04b723386b93bec701aa410e834880401018cdfcc`
- Source item: `RST-P1-023` / `RST-G31`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Static style resolution visited every resolved rule for every arena node. Runtime pseudo-state
application repeated the same full rule scan for every affected tree node. Both paths already had
the terminal node identity needed to reject most rules before the ancestor matcher, but retained
`O(nodes * rules * selector_depth)` rule visits.

## Scope Delivered

- A compiled `ResolvedRuleTerminalIndex` stores each resolved rule under one required terminal
  selector key, prioritized by ID, class, type, state, then host.
- Static resolution builds the index once per resolved rule set and reuses one candidate scratch
  vector for the complete arena traversal.
- Runtime style state retains the index beside the resolved rules and reuses one candidate scratch
  vector for a subtree update.
- Candidate indices are sorted and deduplicated before evaluation, preserving the existing
  specificity/source-order rule vector. The existing full path matcher remains the final oracle for
  compound tokens, ancestor combinators, and fail-closed part selectors.
- The index responsibility and its tests live in `style/rule_index.rs`; `style.rs` remains below the
  repository large-file threshold.

## Deterministic Performance Gate

The ignored release benchmark builds 16,384 class-and-state rules outside the timed region and
selects exactly one candidate. It warms both paths, then records 21 alternating legacy/optimized
sample pairs. The marker includes both raw unsorted sample series, nearest-rank P50/P95, pair order,
rule count, and exact candidate-visit counts so an external validator can recompute the result.

One legacy sample performs 16,384 full selector attempts; one indexed sample performs one full
selector attempt after the terminal lookup, a deterministic 99.994% reduction in matcher visits.
Acceptance requires `optimized_p95_ns * 4 <= legacy_p95_ns`, or at least 75% lower measured P95.
Actual timing values remain pending until the grouped coordinator Cargo batch runs; the structural
visit reduction is not reported as measured speedup.

## Testing Evidence

- The correctness test compares indexed candidates followed by the unchanged full matcher against
  a complete scan across ID, class, type, state, host, ancestor, and nonmatching rules.
- Host and non-host terminal paths are covered separately, while candidate indices must retain the
  original rule vector order.
- `rustfmt +1.94.1` completed for both owned Rust files.
- `git diff --check` completed for both owned Rust files.
- Focused behavior tests, the ignored release benchmark, external marker validation, and package
  checks are pending the multi-task coordinator batch. No Cargo or performance pass is claimed.

## Remaining Scope

This slice closes only the terminal-key prefilter in `RST-P1-023` after validation. Precompiled
ancestor predicates, state-to-rule/node dependency indexing (`RST-P1-024`), typed selector bytecode,
scope/part convergence, computed-style sharing, allocation/RSS qualification, and product-scale
WOC/Editor evidence remain later Runtime73 milestones.
