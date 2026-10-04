---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73/2026-08-22-terminal-selector-index.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/style/rule_index.rs
  - zircon_runtime/src/ui/v2/style/rule_index/hash_bucket_tests.rs
tests:
  - zircon_runtime/src/ui/v2/style/rule_index/hash_bucket_tests.rs
  - tools/tests/test_runtime73_terminal_selector_index_performance_contract.py
---

# Runtime801 · terminal-selector singleton candidate fast path

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime73 terminal selector index | Retain the existing reused-scratch clear, then skip candidate sorting and deduplication when the terminal index yields zero or one candidate; retain the existing order authority for multi-candidate paths. | TDD RED/GREEN source contract `3/3`; lower empty/singleton/order regressions wired; scoped Rustfmt and diff checks pass. Managed Cargo, Release marker, allocator, and product style-latency evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`ResolvedRuleTerminalIndex::collect_candidate_indices` still gathers candidates from the same
terminal buckets and preserves the full selector matcher as the semantic oracle. The pre-existing
caller-owned scratch clear remains before bucket extension; the follow-up guards the existing
`sort_unstable`/`dedup` pair with `candidates.len() > 1`. Empty and singleton buckets therefore
avoid ordering work while the retained scratch-clear behavior prevents stale candidates from
leaking between nodes. Combined universal/class/type/state/host candidates retain the exact
previous sorted-and-deduplicated result.

The lower regression keeps the explicit candidate-order contract and adds singleton and empty
bucket cases, including a reused scratch vector. The source contract locks the clear/fast-path
guard and the existing order regression wiring.

The separately present universal-terminal selector handling and its correctness regression are
outside Runtime801's ownership; this follow-up changes only the singleton ordering branch and
adds the empty/singleton scratch-boundary coverage.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style/rule_index.rs` | `59837E60C5135CE13AF4D5D9234AD3422FCC327924D8AA6E21EACDC6AFF41C26` |
| `zircon_runtime/src/ui/v2/style/rule_index/hash_bucket_tests.rs` | `BD6BD864FCBFF5E9114291437F5E72CC4A48079612A14E20B93FB07F20448F93` |
| `tools/tests/test_runtime73_terminal_selector_index_performance_contract.py` | `5023AE08270600CD9F243008A7A056F9D51A01EDFF442FDC02303C16B7D6C8D4` |

The Runtime801-inclusive combined local source-contract receipt covers `60` modules and `212/212` tests in `0.138s`
with zero failures, errors, or skips. The focused Runtime73 contract now passes `3/3`; a focused
cross-slice rerun of the Runtime73 terminal, Runtime73 prototype, Runtime785, and text-decoration
contracts passes `14/14` in `0.009s`. The asynchronously launched broader non-tooling
performance-contract discovery also completed `2196/2196` tests across `597` modules in `11.205s`;
the focused `14/14` receipt remains the exact current-source evidence for this follow-up's
empty-scratch regression.
Scoped Rustfmt, Python compilation, diff checks, and Wiki validation (`272/272` pages with one
pre-existing metadata warning) also pass. This is intentionally not a measured product speedup:
keep the record `implemented_pending_validation` until the owner-attributed managed batch supplies
lower Rust, ignored Release, allocation, and product p50/p95/p99 evidence. Tooling production
remains deferred.
