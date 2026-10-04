---
title: Runtime206 Secondary Query Postings
category: zircon_runtime
report_id: Runtime206-secondary-query-postings-2026-09-13
date: 2026-09-13
session_id: root-runtime-editor-optimize-20260913
implementation_status: implementation_complete
validation_status: static_passed_managed_cargo_pending
performance_status: deterministic_target_met
---

# Runtime206 Secondary Query Postings

## Scope

`AssetRegistryIndex::get_assets` previously walked the whole UUID map for every
non-type filter, even when a tag, package, or path prefix made the result set
small. Runtime206 P1-041 calls for mutation-owned secondary query indexes. The
existing type posting remains the authoritative fast path for type-qualified
queries; this slice adds tag, package, and path-prefix candidate postings for
the remaining filter path.

## Change

- Add `String -> HashSet<AssetUuid>` postings for tags and package IDs.
- Add an ordered `BTreeMap<String, HashSet<AssetUuid>>` keyed by the canonical
  locator path. A prefix query uses one ordered range walk, so labeled
  subassets sharing a path remain in the same posting without changing URI
  matching semantics.
- Maintain all three postings in `insert_checked` and `remove_source_path`,
  including removal of empty buckets and source removal of labeled entries.
- Select the smallest direct tag/package posting when one exists, borrowing
  its bucket; build and own a path-range UUID union only when no direct posting
  is available, then retain the existing exact tag/path/package predicates and
  canonical URI ordering.
- Use a borrowed `str` range bound for path-prefix lookup so each query does
  not clone its filter prefix into a temporary `String`.
- Keep the public borrowed `Vec<&AssetRegistryEntry>` API and the existing
  type-posting branch unchanged. Compiled filters, visitor/cursor APIs, result
  budgets, and generation leases remain parent-plan work.

## Deterministic Performance Evidence

The source-contract pressure model uses 1,048,576 registry entries uniformly
distributed across 32 tag values. A query for one tag starts from a 32,768-UUID
posting instead of the complete registry.

| Metric | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Candidate entries visited | 1,048,576 | 32,768 | 96.875% |
| Candidate visit ratio | 1.0x | 32.0x fewer | 32.0x |
| Non-matching entries visited | 1,015,808 | 0 in candidate phase | 100.000% |

These are deterministic candidate-count/model results, not CPU-time,
allocator, RSS, or product p50/p95/p99 measurements. The retained result sort
remains `O(K log K)` because the API promises canonical URI order; index
maintenance adds one UUID membership per tag/package/path key.

## Validation

- The initial TDD source contract was red before implementation (`3` of `4`
  checks failed) and green after the fields, mutation maintenance, candidate
  boundary, and pressure model were added. A follow-up borrowed-range guard
  was red before the prefix-bound change and green afterward.
- The focused Runtime206/Runtime85 contract set passes `20/20`, including the
  existing type-posting and root-dedup guards.
- Rust behavior coverage now checks composed tag/package/path-prefix filters,
  labeled subassets, source removal, empty-bucket retirement, and survivor
  preservation in `secondary_query_tests.rs`.
- Rustfmt parse-only checks and scoped whitespace/diff checks pass for the
  touched Runtime sources and tests. Managed Windows Cargo tests and Release
  allocation/latency evidence remain pending under the existing asynchronous
  admission boundary.

The refreshed shared non-tooling Runtime/Editor performance-plus-pressure
loader covers `556` modules and passes `2068/2068` tests in `29.116s`; the
focused Runtime206/Runtime85 plus Editor reference batch passes `37/37` in
`111.865s`. The current all-contract discovery leaves only the deferred WOC
dependency assertion red.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/asset_registry_index.rs` | `8E1478D5C5C23769F241B2DE57C81806C039BFC1D9EF91B8F8A6FCF07024141E` |
| `zircon_runtime/src/asset/registry/query.rs` | `0A867CFBE7029C1BEB92A2A09D337C04E628230E33D15E8579115ECB8A3E7FDF` |
| `zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs` | `B422F76B4A9EDE3F5AE307DB827979B3AB764D56C5F16D72460A093E3E823B51` |
| `tools/tests/test_runtime206_secondary_query_index_performance_contract.py` | `9C964FF6475494E8FB507BFB6CE96143731463890C5C57F145ACA4A97C3CCB1E` |

## Remaining Work

Runtime206 P1-041 is only partially closed: type, tag, package, and path
candidate indexes now exist, but the parent plan still owns compiled query
plans, cursor/visitor and early-stop APIs, result/deadline budgets, generation
leases, unified availability dispositions, large-corpus measurements, and
product query qualification.

The adjacent P1-047 referencer binary-key and bulk-build staging follow-ups
are tracked by the corresponding 2026-09-13 Runtime206 records in this
directory.
