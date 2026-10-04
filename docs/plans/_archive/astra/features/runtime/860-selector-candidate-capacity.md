---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73/2026-09-20-selector-candidate-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/style/rule_index.rs
  - zircon_runtime/src/ui/v2/style/rule_index/candidate_capacity_tests.rs
tests:
  - tools/tests/test_runtime860_selector_candidate_capacity_performance_contract.py
---

# Runtime860 Selector Candidate Scratch Capacity

## Completion entry

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime73 terminal selector scratch | Reserve the summed terminal-bucket upper bound before extending the reused candidate vector, while keeping empty input allocation-free and preserving sorting/deduplication and selector order. | Intentional RED/GREEN source-model contract `4/4`; lower dense/empty regressions and ignored `RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1` marker are wired. A deterministic 4,096-rule model changes geometric growth `13→0`; the combined focused Runtime/Editor batch passes `61/61`, and the post-Runtime860 expanded loader passes `4096/4096` across `968` files in `142.495s`. Managed Cargo/Release, allocator, and selector-style p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

This slice changes only the capacity shape of the caller-owned terminal-index
scratch. It does not change selector parsing, terminal bucket membership,
candidate sort/deduplication, pseudo-state classification, or the full matcher.
Tooling production remains out of scope pending the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style/rule_index.rs` | `65E3E8EC1E9C0DE40D3095B24EE244EB00BA207570A06F26A0F742813B232296` |
| `zircon_runtime/src/ui/v2/style/rule_index/candidate_capacity_tests.rs` | `3FED1CC22947E56DF0149F38F5CFC156EF4513A3440B2EBCC1779BDE00E286A0` |
| `tools/tests/test_runtime860_selector_candidate_capacity_performance_contract.py` | `2C5A25F4A067F8A0175A7B84272F5CCF5055A89486CCF4B56987DDC81D2AD067` |

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until
the combined owner-attributed lane proves current-source compilation, lower
test reachability, allocator behavior, and selector-style product p50/p95/p99
evidence.
