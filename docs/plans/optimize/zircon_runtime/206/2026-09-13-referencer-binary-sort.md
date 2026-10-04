---
title: Runtime206 Referencer Binary Sort Key
category: zircon_runtime
report_id: Runtime206-referencer-binary-sort-2026-09-13
date: 2026-09-13
session_id: root-runtime-editor-optimize-20260913
implementation_status: implementation_complete
validation_status: static_passed_managed_cargo_pending
performance_status: deterministic_target_met
---

# Runtime206 Referencer Binary Sort Key

## Scope

`AssetRegistryIndex::get_referencers_by_uuid` previously used
`sort_by_key(ToString::to_string)`. That preserved display order, but built
owned UUID strings during sorting and left the hot path coupled to formatting.
Runtime206 P1-047 requires a stable binary-key contract for deterministic
referencer ordering.

## Change

- Add `AssetUuid::binary_key()`, returning the UUID's borrowed `[u8; 16]`
  representation without allocation.
- Sort referencer UUIDs with `sort_unstable_by` over that borrowed key.
- Keep the existing UUID set, return type, and deterministic canonical ordering
  contract. Canonical lowercase UUID text and byte order are equivalent for
  this representation; the Rust behavior regression checks that boundary.
- Keep display formatting available for diagnostics and serialization; it is no
  longer part of referencer query ordering.

## Deterministic Performance Evidence

The source-contract pressure model uses 65,536 referencers. It counts the
minimum one owned display-string key per legacy referencer and no owned key in
the binary-key path.

| Metric | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Owned UUID sort keys | 65,536 | 0 | 100.000% |
| Display-format work in query sort | 65,536 keys | 0 keys | 100.000% |

These are deterministic allocation/work-model counts, not CPU-time,
allocator, RSS, or product p50/p95/p99 measurements. The managed Release gate
remains required for acceptance.

## Validation

- TDD source contract was red before implementation (`2` of `3` checks failed)
  and green after the binary key and query sort were added.
- The combined Runtime206 focused source-contract set passes `17/17`, including
  secondary postings, type postings, and the referencer binary-sort guard.
- Rust behavior coverage checks deterministic referencer order alongside the
  secondary posting and source-removal regressions in
  `secondary_query_tests.rs`.
- Rustfmt parse-only and scoped diff checks pass locally. Managed Windows
  Cargo tests and Release allocation/latency evidence remain pending under the
  existing asynchronous admission boundary.

The refreshed shared non-tooling loader covers `556` modules and passes
`2068/2068` tests in `29.116s`; the focused Runtime206/Runtime85 plus
Editor reference batch passes `37/37` in `111.865s`. The all-contract
discovery has only the deferred WOC dependency assertion red.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime_interface/src/resource/asset_uuid.rs` | `03FBB77678F8AB4DF0C3BB9D8B27DB09C79C865D78A267DD535611BE73430B73` |
| `zircon_runtime/src/asset/registry/query.rs` | `0A867CFBE7029C1BEB92A2A09D337C04E628230E33D15E8579115ECB8A3E7FDF` |
| `zircon_runtime/src/asset/registry/asset_registry_index/secondary_query_tests.rs` | `B422F76B4A9EDE3F5AE307DB827979B3AB764D56C5F16D72460A093E3E823B51` |
| `tools/tests/test_runtime206_referencer_binary_sort_performance_contract.py` | `3D074E2C704F85B8F36163B2CA49AF83FC1F95398C5F95B809FFE3C4D56E5F5D` |

## Remaining Work

Runtime206 P1-047 is implemented at the local query boundary but remains
pending managed Cargo/Release acceptance. The parent plan still owns unified
dependency/referencer dispositions, generation leases, cursor/visitor APIs,
large-corpus measurements, and product qualification.
