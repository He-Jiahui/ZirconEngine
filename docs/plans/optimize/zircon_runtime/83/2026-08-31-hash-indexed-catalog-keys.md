# Runtime83 Hash-Indexed Localization Catalog Keys

- Status: `implementation_complete; managed_validation_pending`
- Owner Session: `root-runtime83-localization-path-closeout-20260830`
- Scope: replace private locale/table membership from `BTreeMap` with `HashMap` and catalog key membership from ordered `BTreeSet` with a deduplicating `HashSet`, while retaining the public TOML key projection as an ordered `BTreeSet`.
- Source: `zircon_runtime/src/ui/template/asset/localization/resolve.rs` (`e16b5c621fefbead27caf8c9094501caad915fbcaf100f56cf3a0d80feca945e`)
- Tests: `resolve/performance_tests.rs` (`79ceb63d3bae197c629eeb533943475186d52816369c5358ecd1eaf0b8142c4f`); hash-index guard (`a9afbc925cab7af28ecf6c67b4f8433f7baeee6608c23b66b39b3b7518ae305f`); updated path guard (`a09f62be53b0b5f968d943d96a8ad131750d79460ec49f7bf73af2524c2166e7`).
- Evidence: focused Runtime83 localization contracts `12/12` GREEN; full Runtime performance contracts `980/980` GREEN; `compileall` and `git diff --check` GREEN. The ignored release benchmarks are `RUNTIME83_LOCALIZATION_HASH_KEY_INDEX_BENCH_V1` (10,000 keys) and `RUNTIME83_LOCALIZATION_HASH_TABLE_INDEX_BENCH_V1` (2,048 tables), each with 200,000 probes across 21 alternating pairs; both must report measured legacy/optimized P50 and P95 before acceptance.
- Expected kernel: locale, table, and key membership change from `O(log N)` ordered lookup to average `O(1)` hash lookup; replacement semantics, key filtering, public key ordering, and diagnostic ordering remain behaviorally stable.
