---
title: Runtime206 Registry Build Capacity And Streamed Dependency Bootstrap
category: zircon_runtime
report_id: Runtime206-registry-build-capacity-2026-09-13
date: 2026-09-13
session_id: root-runtime-editor-optimize-20260913
implementation_status: implementation_complete
validation_status: static_passed_managed_cargo_pending
performance_status: deterministic_target_met
---

# Runtime206 Registry Build Capacity And Streamed Dependency Bootstrap

## Scope

`AssetRegistryIndex::from_entries` rebuilt a registry with default hash-table
capacities, then staged every dependency-path vector in a temporary
`Vec<(AssetUuid, Vec<AssetUri>)>` before publishing the same edges into the
reverse index. Large project opens therefore paid repeated hash-table growth,
repeated empty-bucket pruning, and retained an avoidable all-at-once staging
copy.

## Change

- Read the iterator lower bound and reserve the four one-row primary lookup
  maps (`entries_by_uuid`, `uuids_by_path`, `uuid_by_asset_id`, and
  `entry_uuids_by_source`) before insertion.
- Collect only the UUID keys after insertion, then build and publish one
  dependency-path vector at a time. The persistent dependency-path index and
  all dependency/referencer semantics remain unchanged.
- Defer `referencers_by_path` empty-bucket pruning during that streamed phase
  and run it once after the batch; ordinary replacement callers retain the
  previous immediate-pruning behavior.
- Keep duplicate detection, unresolved UUID behavior, and source-removal
  semantics intact.

## Deterministic Performance Evidence

For a 1,048,576-entry rebuild with four dependency paths per entry, the legacy
bootstrap materializes 4,194,304 transient path records before publication;
the streamed path phase materializes at most four transient path records at a
time. The persistent `dependency_paths_by_uuid` index is intentionally
unchanged, so this is a staging-peak model rather than a total resident-memory
claim.

| Metric | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Simultaneously staged transient path records | 4,194,304 | 4 | 99.9999% |
| Primary-map growth phases | repeated rehash | lower-bound reserve | reduced |
| Reverse-path empty-bucket scans during bootstrap | 1,048,576 full scans | 1 final scan | 99.9999% fewer |

The model does not claim CPU time, allocator/RSS, or product open latency; the
managed Windows Release gate remains required.

## Validation

- TDD source contract was red before the initial implementation (`2` of `3`
  checks failed) and green after reservation and streamed bootstrap wiring.
- The follow-up deferred-pruning contract was red before the helper split
  (`1` error in `4` checks) and green after the batch-only pruning path was
  added.
- Rust behavior coverage preserves resolved dependency paths, referencer
  lookup, and unresolved UUID behavior.
- The combined Runtime206 focused source-contract batch passes `17/17` after
  this slice (capacity, referencer, secondary-posting, and type-posting
  contracts).
- Rustfmt parse-only and scoped diff checks pass locally; managed Cargo and
  Release measurements remain pending under the asynchronous admission log.

The refreshed shared non-tooling loader covers `556` modules and passes
`2068/2068` tests in `29.116s`; the focused Runtime206/Runtime85 plus
Editor reference batch passes `37/37` in `111.865s`. The current
all-contract discovery leaves only the deferred WOC dependency assertion
red.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/asset_registry_index.rs` | `8E1478D5C5C23769F241B2DE57C81806C039BFC1D9EF91B8F8A6FCF07024141E` |
| `zircon_runtime/src/asset/registry/asset_registry_index/build_tests.rs` | `A3B33D2A638224EBFEE058D9D7104566760CBBCBF8FB840CC81FBBF9AFC3B23D` |
| `tools/tests/test_runtime206_registry_build_capacity_performance_contract.py` | `C7743CCF5039028250CAF53BF6BA1DCF52EAD5F8AE6EB55174A3C471CE86F72A` |

## Remaining Work

This closes the bounded bulk-build staging optimization only. Runtime206 still
owns compiled query/cursor APIs, generation leases, currentness validation,
unified dispositions, large-corpus measurements, and product qualification.
