---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/218-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/718-hierarchy-filter-lazy-match-flags.md
  - docs/plans/astra/features/editor/719-hierarchy-filter-test-contract-repair.md
related_code:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
tests:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
---

# Editor Hierarchy Filter Borrowed Query and Output Capacity

## Scope

The retained hierarchy filter already had an ASCII byte matcher, but the
projection boundary still allocated a lowercase `String` for every non-empty
query. Its ancestry stack also grew from an empty vector, and the filtered row
output relied on `filter_map().collect()` without knowing the final row count.

The filter now keeps ASCII queries borrowed in a `Cow<str>` and only owns a
normalized query for the non-ASCII path. When a Unicode name is searched with
an ASCII query, the matcher lowercases the name once and compares only
character-boundary slices with `eq_ignore_ascii_case`, preserving the previous
case-insensitive behavior without rebuilding the query. Name matching now runs
before parent-index construction, so a no-result query exits without building
the ancestry index or its match-flag vector. The flags are materialized only on
the first match, the ancestry stack is reserved to the source row count, and
the output vector is allocated once at the exact match-plus-ancestor count.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor01 / Editor129 / Editor181 / Editor218 / EUI-716/718 | Borrow ASCII hierarchy queries, lazily materialize match flags, skip parent-index work for no-result queries, preserve Unicode fallback semantics, and remove ancestry/output vector growth | implemented_pending_validation | Rust source/behavior regressions cover uppercase ASCII queries against Unicode lowercase mappings and the no-match short circuit; standalone semantic/source probes pass. Scoped Rustfmt and hierarchy contracts pass; managed Rust and Windows Release filter CPU/allocation/p95 evidence remain pending. |

## Complexity and allocation boundary

The filter remains an intentional full `O(N)` hierarchy scan while the product
has no generation-owned query index. For ASCII queries, query-normalization
allocation changes from one owned buffer to zero; no-result queries also avoid
parent-index allocation and traversal. When results exist, the first match
allocates the `O(N)` flags once, ancestry bookkeeping reserves its `O(N)` stack
once, and the result vector performs one allocation for the known `M` visible rows
instead of geometric growth. Unicode queries retain the required owned
normalization and Unicode-name fallback. No row ordering, ancestor retention,
selection overlay, or empty-query behavior changes.

## Static evidence

- The in-file regression exercises ASCII and Unicode case-insensitive matching,
  including Kelvin-sign and dotted-I lowercase mappings, and verifies that a
  no-match query exits before parent-index construction; managed Rust execution
  remains pending.
- A standalone Rust type check confirms the `Cow<str>` branch and the
  character-boundary Unicode fallback compile on Rust 1.94-compatible syntax.
- The batched Editor hierarchy projection/viewport/native-authority contracts
  pass `31/31` in one invocation after this change; scoped Rustfmt and
  `git diff --check` pass.
- The existing hierarchy-filter metrics Pester contract passes `6/6`; its two
  missing-counter warnings are the intentional negative fixtures.
- The subsequent single-invocation non-tooling Runtime/Editor performance-plus-pressure loader
  covered 343 modules and passed `1320/1320` tests in `79.827s`.
- After the lazy match-flag extension, that same batched loader reran across 343 modules and
  passed `1320/1320` in `10.083s`; this remains source/contract evidence only.
- The final current-source rerun, including the Runtime720/721 and Editor722 slices, covered the
  same 343 modules and passed `1320/1320` in `94.188s`.
- Editor719 aligned the in-file no-match source assertion with the lazy `Option<Vec<bool>>`
  branch; the focused Editor product-interaction contract batch passed `10/10` afterward.
- These checks are source/model evidence only. They do not claim product CPU,
  allocator, RSS, or p50/p95/p99 acceptance.

## Managed gate

No new coordinator request was created for this focused slice. It joins the
existing Runtime/Editor asynchronous batch. The record remains
`implemented_pending_validation` until an owner-attributed Windows Release run
verifies hierarchy-filter allocation counts and input-to-damage p50/p95/p99 at
the declared 10k/100k workloads.
