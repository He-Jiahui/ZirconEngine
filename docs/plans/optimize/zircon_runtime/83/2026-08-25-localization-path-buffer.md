Plan: docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md
Milestone: M8
Status: completed
Files: ["zircon_runtime/src/ui/template/asset/localization/collect.rs", "zircon_runtime/src/ui/template/asset/localization/collect/performance_tests.rs", "zircon_runtime/src/ui/template/asset/localization/resolve.rs", "zircon_runtime/src/ui/template/asset/localization/resolve/performance_tests.rs", "tools/tests/test_runtime83_localization_path_performance_contract.py"]

# Runtime83 Localization Path Buffer And Locale Lookup Batch

## Scope delivered

This batch removes temporary path allocations from two localization traversals and hoists the
locale-level catalog lookup out of the per-dependency loop. It preserves emitted paths, sorted
reports, diagnostic codes and messages, locale trimming, fallback severity, and TOML leaf keys.

- TOML key flattening now grows and truncates one `String`; only accepted leaf keys are cloned
  into the returned `BTreeSet`.
- document localization collection now reuses one path buffer across nodes, stylesheet rules,
  nested tables, and arrays; only paths retained by the report are cloned.
- catalog validation borrows the selected locale table map once per report instead of probing the
  outer `BTreeMap` for every dependency.

The parent plan remains broader than this performance slice. Runtime localization service,
compiled catalogs, fallback chains, message formatting, culture switching, product migration,
and the complete M8 fault/scale matrix remain open.

## Fresh testing evidence

TDD first established three failing source contracts against the allocation-heavy implementation.
After the change, all three pass, Python bytecode compilation passes, and Rust 1.94.1
`rustfmt --check` passes for both production modules.

Five process-level repetitions of the isolated optimized-vs-legacy Rust benchmark produced these
median-of-run nearest-rank values:

| workload | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| 10k localization leaf paths, P50 | 3.7068 ms | 2.3588 ms | 36.366% |
| 10k localization leaf paths, P95 | 5.8749 ms | 3.8554 ms | 34.375% |
| path-construction allocations | 33,876 | 11,669 | 65.554% |
| 50k catalog dependencies, P50 | 6.0446 ms | 2.9163 ms | 51.754% |
| 50k catalog dependencies, P95 | 7.7679 ms | 4.8715 ms | 37.287% |
| outer locale-map lookups | 50,000 | 1 | 99.998% |

The managed Windows validation batch will rerun the source contracts, focused localization
behavior tests, formatting, and release benchmark before integration. No local Cargo command or
Cargo dry-run was launched.

## Review

The change is deliberately limited to private localization traversal and lookup helpers. It adds
no public API, compatibility shim, cache, global mutable state, or behavior policy. Independent
review remains an integration gate after managed validation returns.
